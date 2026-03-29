//! Filter — expression parser and evaluator for filtering table lines.
//!
//! Implements the `scols-filter(5)` expression language. Supports:
//! - Boolean operators: `&&` / `AND`, `||` / `OR`, `!` / `NOT`
//! - Comparisons: `==`, `!=`, `<`, `<=`, `>`, `>=` (and word forms)
//! - Regex matching: `=~`, `!~`
//! - Literals: integers (with K/M/G/T suffixes), floats, quoted strings,
//!   booleans (`true`/`false`)
//! - Holders: unquoted column name references
//!
//! # Example
//!
//! ```
//! use smartcols::Filter;
//!
//! let filter = Filter::parse(r#"SIZE > 1G && TYPE == "disk""#).unwrap();
//! let result = filter.evaluate(|col| match col {
//!     "SIZE" => Some("2147483648".into()),
//!     "TYPE" => Some("disk".into()),
//!     _ => None,
//! }).unwrap();
//! assert!(result);
//! ```
//!
//! Operator precedence (lowest to highest): `OR`, `AND`, comparisons, `NOT`.

use regex::Regex;
use std::fmt;

/// A parsed filter expression, ready to evaluate against line data.
#[derive(Debug, Clone)]
pub struct Filter {
    root: Expr,
}

/// AST node for a filter expression.
#[derive(Debug, Clone)]
pub enum Expr {
    // Logical operators
    And(Box<Expr>, Box<Expr>),
    Or(Box<Expr>, Box<Expr>),
    Not(Box<Expr>),

    // Comparison operators
    Eq(Box<Expr>, Box<Expr>),
    Ne(Box<Expr>, Box<Expr>),
    Lt(Box<Expr>, Box<Expr>),
    Le(Box<Expr>, Box<Expr>),
    Gt(Box<Expr>, Box<Expr>),
    Ge(Box<Expr>, Box<Expr>),

    // Regex match / not-match
    RegexMatch(Box<Expr>, Box<Expr>),
    RegexNotMatch(Box<Expr>, Box<Expr>),

    // Leaf nodes
    Holder(String), // column name reference
    Integer(u64),
    Float(f64),
    Str(String),
    Bool(bool),
}

/// A value produced during filter evaluation.
#[derive(Debug, Clone)]
pub enum Value {
    Integer(u64),
    Float(f64),
    Str(String),
    Bool(bool),
}

/// Errors from parsing or evaluating a filter.
#[derive(Debug, Clone)]
pub struct FilterError {
    pub message: String,
}

impl fmt::Display for FilterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "filter error: {}", self.message)
    }
}

impl std::error::Error for FilterError {}

impl Filter {
    /// Parse a filter expression string.
    pub fn parse(input: &str) -> Result<Self, FilterError> {
        let tokens = tokenize(input)?;
        let mut pos = 0;
        let root = parse_or(&tokens, &mut pos)?;
        if pos < tokens.len() {
            return Err(FilterError {
                message: format!("unexpected token: {:?}", tokens[pos]),
            });
        }
        Ok(Self { root })
    }

    /// Evaluate the filter against a line, using a resolver function that maps
    /// column names to their string values.
    pub fn evaluate<F>(&self, resolve: F) -> Result<bool, FilterError>
    where
        F: Fn(&str) -> Option<String>,
    {
        let val = eval_expr(&self.root, &resolve)?;
        Ok(value_to_bool(&val))
    }

    /// Return all holder (column) names referenced in this filter.
    pub fn holders(&self) -> Vec<String> {
        let mut names = Vec::new();
        collect_holders(&self.root, &mut names);
        names.sort();
        names.dedup();
        names
    }
}

fn collect_holders(expr: &Expr, out: &mut Vec<String>) {
    match expr {
        Expr::Holder(name) => out.push(name.clone()),
        Expr::And(a, b)
        | Expr::Or(a, b)
        | Expr::Eq(a, b)
        | Expr::Ne(a, b)
        | Expr::Lt(a, b)
        | Expr::Le(a, b)
        | Expr::Gt(a, b)
        | Expr::Ge(a, b)
        | Expr::RegexMatch(a, b)
        | Expr::RegexNotMatch(a, b) => {
            collect_holders(a, out);
            collect_holders(b, out);
        }
        Expr::Not(a) => collect_holders(a, out),
        Expr::Integer(_) | Expr::Float(_) | Expr::Str(_) | Expr::Bool(_) => {}
    }
}

fn eval_expr<F>(expr: &Expr, resolve: &F) -> Result<Value, FilterError>
where
    F: Fn(&str) -> Option<String>,
{
    match expr {
        Expr::Integer(n) => Ok(Value::Integer(*n)),
        Expr::Float(f) => Ok(Value::Float(*f)),
        Expr::Str(s) => Ok(Value::Str(s.clone())),
        Expr::Bool(b) => Ok(Value::Bool(*b)),
        Expr::Holder(name) => {
            let data = resolve(name).unwrap_or_default();
            Ok(Value::Str(data))
        }

        Expr::Not(inner) => {
            let val = eval_expr(inner, resolve)?;
            Ok(Value::Bool(!value_to_bool(&val)))
        }

        Expr::And(a, b) => {
            let va = eval_expr(a, resolve)?;
            if !value_to_bool(&va) {
                return Ok(Value::Bool(false));
            }
            let vb = eval_expr(b, resolve)?;
            Ok(Value::Bool(value_to_bool(&vb)))
        }

        Expr::Or(a, b) => {
            let va = eval_expr(a, resolve)?;
            if value_to_bool(&va) {
                return Ok(Value::Bool(true));
            }
            let vb = eval_expr(b, resolve)?;
            Ok(Value::Bool(value_to_bool(&vb)))
        }

        Expr::Eq(a, b) => {
            compare(a, b, resolve, |ord| ord == std::cmp::Ordering::Equal)
        }
        Expr::Ne(a, b) => {
            compare(a, b, resolve, |ord| ord != std::cmp::Ordering::Equal)
        }
        Expr::Lt(a, b) => {
            compare(a, b, resolve, |ord| ord == std::cmp::Ordering::Less)
        }
        Expr::Le(a, b) => compare(a, b, resolve, |ord| {
            ord == std::cmp::Ordering::Less || ord == std::cmp::Ordering::Equal
        }),
        Expr::Gt(a, b) => {
            compare(a, b, resolve, |ord| ord == std::cmp::Ordering::Greater)
        }
        Expr::Ge(a, b) => compare(a, b, resolve, |ord| {
            ord == std::cmp::Ordering::Greater
                || ord == std::cmp::Ordering::Equal
        }),

        Expr::RegexMatch(a, b) => regex_op(a, b, resolve, false),
        Expr::RegexNotMatch(a, b) => regex_op(a, b, resolve, true),
    }
}

fn compare<F, P>(
    a: &Expr,
    b: &Expr,
    resolve: &F,
    pred: P,
) -> Result<Value, FilterError>
where
    F: Fn(&str) -> Option<String>,
    P: Fn(std::cmp::Ordering) -> bool,
{
    let va = eval_expr(a, resolve)?;
    let vb = eval_expr(b, resolve)?;

    // Coerce to common type: prefer the literal's type; if mixed
    // int/float, promote to float.
    let ord = match (&va, &vb) {
        (Value::Integer(a), Value::Integer(b)) => a.cmp(b),
        (Value::Float(a), Value::Float(b)) => {
            a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal)
        }
        (Value::Integer(a), Value::Float(b)) => (*a as f64)
            .partial_cmp(b)
            .unwrap_or(std::cmp::Ordering::Equal),
        (Value::Float(a), Value::Integer(b)) => a
            .partial_cmp(&(*b as f64))
            .unwrap_or(std::cmp::Ordering::Equal),
        (Value::Bool(a), Value::Bool(b)) => a.cmp(b),
        // For string comparisons, try numeric coercion first.
        (Value::Str(a), Value::Integer(b)) => {
            if let Some(n) = parse_integer_value(a) {
                n.cmp(b)
            } else {
                a.as_str().cmp(b.to_string().as_str())
            }
        }
        (Value::Integer(a), Value::Str(b)) => {
            if let Some(n) = parse_integer_value(b) {
                a.cmp(&n)
            } else {
                a.to_string().as_str().cmp(b.as_str())
            }
        }
        (Value::Str(a), Value::Float(b)) => {
            if let Ok(f) = a.parse::<f64>() {
                f.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal)
            } else {
                a.as_str().cmp(b.to_string().as_str())
            }
        }
        (Value::Float(a), Value::Str(b)) => {
            if let Ok(f) = b.parse::<f64>() {
                a.partial_cmp(&f).unwrap_or(std::cmp::Ordering::Equal)
            } else {
                a.to_string().as_str().cmp(b.as_str())
            }
        }
        // Default: string comparison.
        _ => value_to_string(&va).cmp(&value_to_string(&vb)),
    };

    Ok(Value::Bool(pred(ord)))
}

fn regex_op<F>(
    a: &Expr,
    b: &Expr,
    resolve: &F,
    negate: bool,
) -> Result<Value, FilterError>
where
    F: Fn(&str) -> Option<String>,
{
    let va = eval_expr(a, resolve)?;
    let vb = eval_expr(b, resolve)?;
    let text = value_to_string(&va);
    let pattern = value_to_string(&vb);

    let re = Regex::new(&pattern).map_err(|e| FilterError {
        message: format!("invalid regex '{pattern}': {e}"),
    })?;

    let matched = re.is_match(&text);
    Ok(Value::Bool(if negate { !matched } else { matched }))
}

fn value_to_bool(v: &Value) -> bool {
    match v {
        Value::Bool(b) => *b,
        Value::Integer(n) => *n != 0,
        Value::Float(f) => *f != 0.0,
        Value::Str(s) => !s.is_empty(),
    }
}

fn value_to_string(v: &Value) -> String {
    match v {
        Value::Bool(b) => b.to_string(),
        Value::Integer(n) => n.to_string(),
        Value::Float(f) => f.to_string(),
        Value::Str(s) => s.clone(),
    }
}

#[derive(Debug, Clone, PartialEq)]
enum Token {
    LParen,
    RParen,
    And,
    Or,
    Not,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    RegexMatch,
    RegexNotMatch,
    Integer(u64),
    Float(f64),
    Str(String),
    Bool(bool),
    Holder(String),
}

fn tokenize(input: &str) -> Result<Vec<Token>, FilterError> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        // Skip whitespace.
        if chars[i].is_ascii_whitespace() {
            i += 1;
            continue;
        }

        // Two-character operators.
        if i + 1 < chars.len() {
            let two: String = chars[i..i + 2].iter().collect();
            match two.as_str() {
                "&&" => {
                    tokens.push(Token::And);
                    i += 2;
                    continue;
                }
                "||" => {
                    tokens.push(Token::Or);
                    i += 2;
                    continue;
                }
                "==" => {
                    tokens.push(Token::Eq);
                    i += 2;
                    continue;
                }
                "!=" => {
                    tokens.push(Token::Ne);
                    i += 2;
                    continue;
                }
                ">=" => {
                    tokens.push(Token::Ge);
                    i += 2;
                    continue;
                }
                "<=" => {
                    tokens.push(Token::Le);
                    i += 2;
                    continue;
                }
                "=~" => {
                    tokens.push(Token::RegexMatch);
                    i += 2;
                    continue;
                }
                "!~" => {
                    tokens.push(Token::RegexNotMatch);
                    i += 2;
                    continue;
                }
                _ => {}
            }
        }

        // Single-character operators/punctuation.
        match chars[i] {
            '(' => {
                tokens.push(Token::LParen);
                i += 1;
                continue;
            }
            ')' => {
                tokens.push(Token::RParen);
                i += 1;
                continue;
            }
            '!' => {
                tokens.push(Token::Not);
                i += 1;
                continue;
            }
            '>' => {
                tokens.push(Token::Gt);
                i += 1;
                continue;
            }
            '<' => {
                tokens.push(Token::Lt);
                i += 1;
                continue;
            }
            _ => {}
        }

        // Quoted strings.
        if chars[i] == '"' || chars[i] == '\'' {
            let quote = chars[i];
            i += 1;
            let mut s = String::new();
            while i < chars.len() && chars[i] != quote {
                if chars[i] == '\n' {
                    return Err(FilterError {
                        message: "unterminated string (newline in string)"
                            .to_string(),
                    });
                }
                s.push(chars[i]);
                i += 1;
            }
            if i >= chars.len() {
                return Err(FilterError {
                    message: "unterminated string".to_string(),
                });
            }
            i += 1; // closing quote
            tokens.push(Token::Str(s));
            continue;
        }

        // Numbers: [0-9]+ possibly followed by a decimal or suffix.
        if chars[i].is_ascii_digit() {
            let start = i;
            while i < chars.len() && chars[i].is_ascii_digit() {
                i += 1;
            }

            // Check for float (decimal point followed by digits).
            if i < chars.len()
                && chars[i] == '.'
                && i + 1 < chars.len()
                && chars[i + 1].is_ascii_digit()
            {
                i += 1; // skip '.'
                while i < chars.len() && chars[i].is_ascii_digit() {
                    i += 1;
                }
                let num_str: String = chars[start..i].iter().collect();
                let f = num_str.parse::<f64>().map_err(|e| FilterError {
                    message: format!("invalid float '{num_str}': {e}"),
                })?;
                tokens.push(Token::Float(f));
                continue;
            }

            let num_str: String = chars[start..i].iter().collect();
            let base: u64 = num_str.parse().map_err(|e| FilterError {
                message: format!("invalid integer '{num_str}': {e}"),
            })?;

            // Check for suffix: K, M, G, T, P, E, Z, Y (optionally followed by "iB").
            if i < chars.len() && "KMGTPEZY".contains(chars[i]) {
                let multiplier = match chars[i] {
                    'K' => 1024u64,
                    'M' => 1024u64.pow(2),
                    'G' => 1024u64.pow(3),
                    'T' => 1024u64.pow(4),
                    'P' => 1024u64.pow(5),
                    'E' => 1024u64.pow(6),
                    'Z' => 1024u64.checked_pow(7).unwrap_or(u64::MAX),
                    'Y' => 1024u64.checked_pow(8).unwrap_or(u64::MAX),
                    _ => unreachable!(),
                };
                i += 1;
                // Skip optional "iB".
                if i + 1 < chars.len() && chars[i] == 'i' && chars[i + 1] == 'B'
                {
                    i += 2;
                }
                tokens.push(Token::Integer(base.saturating_mul(multiplier)));
            } else {
                tokens.push(Token::Integer(base));
            }
            continue;
        }

        // Identifiers: keywords (AND, OR, NOT, true, false) or holders.
        if chars[i].is_ascii_alphabetic() {
            let start = i;
            while i < chars.len()
                && (chars[i].is_ascii_alphanumeric()
                    || "_.%:/-".contains(chars[i]))
            {
                i += 1;
            }
            let word: String = chars[start..i].iter().collect();
            match word.as_str() {
                "AND" => tokens.push(Token::And),
                "OR" => tokens.push(Token::Or),
                "NOT" => tokens.push(Token::Not),
                "EQ" => tokens.push(Token::Eq),
                "NE" => tokens.push(Token::Ne),
                "LT" => tokens.push(Token::Lt),
                "LE" => tokens.push(Token::Le),
                "GT" => tokens.push(Token::Gt),
                "GE" => tokens.push(Token::Ge),
                "true" | "TRUE" => tokens.push(Token::Bool(true)),
                "false" | "FALSE" => tokens.push(Token::Bool(false)),
                _ => tokens.push(Token::Holder(word)),
            }
            continue;
        }

        return Err(FilterError {
            message: format!("unexpected character: {:?}", chars[i]),
        });
    }

    Ok(tokens)
}

// Precedence (lowest to highest):
//   or
//   and
//   comparison (==, !=, <, <=, >, >=, =~, !~)
//   not (unary)
//   atom (literal, holder, parenthesized expr)

fn parse_or(tokens: &[Token], pos: &mut usize) -> Result<Expr, FilterError> {
    let mut left = parse_and(tokens, pos)?;
    while *pos < tokens.len() && tokens[*pos] == Token::Or {
        *pos += 1;
        let right = parse_and(tokens, pos)?;
        left = Expr::Or(Box::new(left), Box::new(right));
    }
    Ok(left)
}

fn parse_and(tokens: &[Token], pos: &mut usize) -> Result<Expr, FilterError> {
    let mut left = parse_comparison(tokens, pos)?;
    while *pos < tokens.len() && tokens[*pos] == Token::And {
        *pos += 1;
        let right = parse_comparison(tokens, pos)?;
        left = Expr::And(Box::new(left), Box::new(right));
    }
    Ok(left)
}

fn parse_comparison(
    tokens: &[Token],
    pos: &mut usize,
) -> Result<Expr, FilterError> {
    let left = parse_not(tokens, pos)?;

    if *pos >= tokens.len() {
        return Ok(left);
    }

    let op = &tokens[*pos];
    match op {
        Token::Eq
        | Token::Ne
        | Token::Lt
        | Token::Le
        | Token::Gt
        | Token::Ge
        | Token::RegexMatch
        | Token::RegexNotMatch => {
            let op = tokens[*pos].clone();
            *pos += 1;
            let right = parse_not(tokens, pos)?;
            let (left, right) = (Box::new(left), Box::new(right));
            Ok(match op {
                Token::Eq => Expr::Eq(left, right),
                Token::Ne => Expr::Ne(left, right),
                Token::Lt => Expr::Lt(left, right),
                Token::Le => Expr::Le(left, right),
                Token::Gt => Expr::Gt(left, right),
                Token::Ge => Expr::Ge(left, right),
                Token::RegexMatch => Expr::RegexMatch(left, right),
                Token::RegexNotMatch => Expr::RegexNotMatch(left, right),
                _ => unreachable!(),
            })
        }
        _ => Ok(left),
    }
}

fn parse_not(tokens: &[Token], pos: &mut usize) -> Result<Expr, FilterError> {
    if *pos < tokens.len() && tokens[*pos] == Token::Not {
        *pos += 1;
        let inner = parse_not(tokens, pos)?;
        return Ok(Expr::Not(Box::new(inner)));
    }
    parse_atom(tokens, pos)
}

fn parse_atom(tokens: &[Token], pos: &mut usize) -> Result<Expr, FilterError> {
    if *pos >= tokens.len() {
        return Err(FilterError {
            message: "unexpected end of expression".to_string(),
        });
    }

    match &tokens[*pos] {
        Token::LParen => {
            *pos += 1;
            let expr = parse_or(tokens, pos)?;
            if *pos >= tokens.len() || tokens[*pos] != Token::RParen {
                return Err(FilterError {
                    message: "expected ')'".to_string(),
                });
            }
            *pos += 1;
            Ok(expr)
        }
        Token::Integer(n) => {
            let n = *n;
            *pos += 1;
            Ok(Expr::Integer(n))
        }
        Token::Float(f) => {
            let f = *f;
            *pos += 1;
            Ok(Expr::Float(f))
        }
        Token::Str(s) => {
            let s = s.clone();
            *pos += 1;
            Ok(Expr::Str(s))
        }
        Token::Bool(b) => {
            let b = *b;
            *pos += 1;
            Ok(Expr::Bool(b))
        }
        Token::Holder(name) => {
            let name = name.clone();
            *pos += 1;
            Ok(Expr::Holder(name))
        }
        other => Err(FilterError {
            message: format!("unexpected token: {other:?}"),
        }),
    }
}

fn parse_integer_value(s: &str) -> Option<u64> {
    // Try plain parse first.
    if let Ok(n) = s.parse::<u64>() {
        return Some(n);
    }
    // Try with suffix.
    let s = s.trim();
    if s.is_empty() {
        return None;
    }
    // Find where digits end.
    let digit_end = s.find(|c: char| !c.is_ascii_digit())?;
    let (num_part, suffix) = s.split_at(digit_end);
    let base: u64 = num_part.parse().ok()?;
    let suffix = suffix.strip_suffix("iB").unwrap_or(suffix);
    let multiplier = match suffix {
        "K" => 1024u64,
        "M" => 1024u64.pow(2),
        "G" => 1024u64.pow(3),
        "T" => 1024u64.pow(4),
        "P" => 1024u64.pow(5),
        "E" => 1024u64.pow(6),
        _ => return None,
    };
    Some(base.saturating_mul(multiplier))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn eval(expr: &str, resolve: impl Fn(&str) -> Option<String>) -> bool {
        let filter = Filter::parse(expr).unwrap();
        filter.evaluate(resolve).unwrap()
    }

    fn no_data(_name: &str) -> Option<String> {
        None
    }

    #[test]
    fn simple_string_eq() {
        assert!(eval(r#"NAME == "sda""#, |name| {
            if name == "NAME" {
                Some("sda".into())
            } else {
                None
            }
        }));
    }

    #[test]
    fn simple_string_ne() {
        assert!(eval(r#"NAME != "sdb""#, |name| {
            if name == "NAME" {
                Some("sda".into())
            } else {
                None
            }
        }));
    }

    #[test]
    fn integer_comparison() {
        assert!(eval("SIZE > 100", |name| {
            if name == "SIZE" {
                Some("200".into())
            } else {
                None
            }
        }));
        assert!(!eval("SIZE > 100", |name| {
            if name == "SIZE" {
                Some("50".into())
            } else {
                None
            }
        }));
    }

    #[test]
    fn integer_suffix() {
        // 2K = 2048
        let filter = Filter::parse("SIZE > 1K").unwrap();
        assert!(
            filter
                .evaluate(|name| {
                    if name == "SIZE" {
                        Some("2048".into())
                    } else {
                        None
                    }
                })
                .unwrap()
        );
    }

    #[test]
    fn boolean_and_or() {
        assert!(eval(r#"NAME == "sda" && SIZE > 100"#, |name| match name {
            "NAME" => Some("sda".into()),
            "SIZE" => Some("200".into()),
            _ => None,
        }));
        assert!(!eval(r#"NAME == "sda" && SIZE > 100"#, |name| match name {
            "NAME" => Some("sdb".into()),
            "SIZE" => Some("200".into()),
            _ => None,
        }));
        assert!(eval(r#"NAME == "sda" || NAME == "sdb""#, |name| {
            if name == "NAME" {
                Some("sdb".into())
            } else {
                None
            }
        }));
    }

    #[test]
    fn not_operator() {
        // NOT has highest precedence: NOT NAME == "sda" means (NOT NAME) == "sda"
        // Use parens for the "negate comparison" case.
        assert!(eval(r#"NOT (NAME == "sda")"#, |name| {
            if name == "NAME" {
                Some("sdb".into())
            } else {
                None
            }
        }));
        assert!(eval(r#"!(NAME == "sda")"#, |name| {
            if name == "NAME" {
                Some("sdb".into())
            } else {
                None
            }
        }));
        // Bare NOT on a truthy value.
        assert!(!eval("NOT NAME", |name| {
            if name == "NAME" {
                Some("hello".into())
            } else {
                None
            }
        }));
        assert!(eval("NOT NAME", no_data));
    }

    #[test]
    fn regex_match() {
        assert!(eval(r#"NAME =~ "sd[ab]""#, |name| {
            if name == "NAME" {
                Some("sda".into())
            } else {
                None
            }
        }));
        assert!(!eval(r#"NAME !~ "sd[ab]""#, |name| {
            if name == "NAME" {
                Some("sda".into())
            } else {
                None
            }
        }));
    }

    #[test]
    fn parenthesized() {
        assert!(eval(
            r#"(NAME == "sda" || NAME == "sdb") && SIZE > 100"#,
            |name| match name {
                "NAME" => Some("sdb".into()),
                "SIZE" => Some("200".into()),
                _ => None,
            }
        ));
    }

    #[test]
    fn holder_truthiness() {
        // A bare holder is true if non-empty.
        assert!(eval("NAME", |name| {
            if name == "NAME" {
                Some("hello".into())
            } else {
                None
            }
        }));
        assert!(!eval("NAME", no_data));
    }

    #[test]
    fn holders_extraction() {
        let filter = Filter::parse(r#"NAME == "sda" && SIZE > 100"#).unwrap();
        let holders = filter.holders();
        assert_eq!(holders, vec!["NAME", "SIZE"]);
    }

    #[test]
    fn float_comparison() {
        assert!(eval("LOAD > 1.5", |name| {
            if name == "LOAD" {
                Some("2.0".into())
            } else {
                None
            }
        }));
    }

    #[test]
    fn boolean_literal() {
        assert!(eval("true", no_data));
        assert!(!eval("false", no_data));
    }

    #[test]
    fn word_operators() {
        assert!(eval(r#"NAME EQ "sda""#, |name| {
            if name == "NAME" {
                Some("sda".into())
            } else {
                None
            }
        }));
        assert!(eval(
            r#"NAME EQ "sda" AND SIZE GT 100"#,
            |name| match name {
                "NAME" => Some("sda".into()),
                "SIZE" => Some("200".into()),
                _ => None,
            }
        ));
    }
}
