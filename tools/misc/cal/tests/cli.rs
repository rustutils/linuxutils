use assert_cmd::Command;
use predicates::prelude::*;

fn cal() -> Command {
    Command::cargo_bin("cal").unwrap()
}

#[test]
fn january_2024_header() {
    cal()
        .args(["1", "2024"])
        .assert()
        .success()
        .stdout(predicate::str::contains("January 2024"));
}

#[test]
fn january_2024_day_layout() {
    // Jan 1 2024 is a Monday. With Sunday-first (default), the 1 should
    // appear in the Monday column (second position).
    let output = cal().args(["1", "2024"]).output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    // Header line should be Sunday-first by default
    assert!(stdout.contains("Su Mo Tu We Th Fr Sa"));

    // The first week line should have the 1 in the Monday column.
    // With Sunday-first, Monday is position 2, so the line starts with
    // blanks for Sunday, then " 1".
    let lines: Vec<&str> = stdout.lines().collect();
    // Find the first line with day numbers (after header lines)
    let first_week = lines.iter().find(|l| l.trim().starts_with('1')).unwrap();
    // "   1" — 3 spaces (Sunday blank) then 1
    assert!(
        first_week.starts_with("    1"),
        "Expected first week to start with blank Sunday then 1, got: '{first_week}'"
    );
}

#[test]
fn january_2024_has_31_days() {
    cal()
        .args(["1", "2024"])
        .assert()
        .success()
        .stdout(predicate::str::contains("31"));
}

#[test]
fn february_2024_leap_year() {
    // 2024 is a leap year, February should have 29 days
    cal()
        .args(["2", "2024"])
        .assert()
        .success()
        .stdout(predicate::str::contains("February 2024"))
        .stdout(predicate::str::contains("29"));
}

#[test]
fn february_2023_not_leap_year() {
    // 2023 is not a leap year, February should have 28 days but not 29
    let output = cal().args(["2", "2023"]).output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert!(stdout.contains("February 2023"));
    assert!(stdout.contains("28"));
    // Should not contain 29 as a day number. Check that "29" doesn't appear
    // in any day position.
    let has_29_as_day = stdout.lines().any(|line| {
        // Skip header lines
        if line.contains("February")
            || line.contains("Su")
            || line.contains("Mo Tu")
        {
            return false;
        }
        // Check for 29 as a standalone number in the calendar grid
        line.split_whitespace().any(|tok| tok == "29")
    });
    assert!(!has_29_as_day, "February 2023 should not contain day 29");
}

#[test]
fn full_year_2024() {
    // `cal -y 2024` should show all 12 months
    let output = cal().args(["-y", "2024"]).output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert!(stdout.contains("2024"));
    assert!(stdout.contains("January"));
    assert!(stdout.contains("February"));
    assert!(stdout.contains("March"));
    assert!(stdout.contains("April"));
    assert!(stdout.contains("May"));
    assert!(stdout.contains("June"));
    assert!(stdout.contains("July"));
    assert!(stdout.contains("August"));
    assert!(stdout.contains("September"));
    assert!(stdout.contains("October"));
    assert!(stdout.contains("November"));
    assert!(stdout.contains("December"));
}

#[test]
fn year_only_shows_full_year() {
    // A single numeric argument is treated as a year
    let output = cal().args(["2024"]).output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert!(stdout.contains("January"));
    assert!(stdout.contains("December"));
}

#[test]
fn monday_first() {
    // -m flag should show Monday as first day of the week
    cal()
        .args(["-m", "1", "2024"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Mo Tu We Th Fr Sa Su"));
}

#[test]
fn monday_first_day_alignment() {
    // With Monday-first, Jan 1 2024 (Monday) should be in the first column
    let output = cal().args(["-m", "1", "2024"]).output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    let lines: Vec<&str> = stdout.lines().collect();
    let first_week = lines.iter().find(|l| l.trim().starts_with('1')).unwrap();
    // With Monday first, Jan 1 (Monday) should be in column 1, no leading blanks
    assert!(
        first_week.starts_with(" 1"),
        "Expected first week to start with ' 1' (Monday column), got: '{first_week}'"
    );
}

#[test]
fn sunday_first_explicit() {
    // -s flag should show Sunday as first day of the week (default, but explicit)
    cal()
        .args(["-s", "1", "2024"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Su Mo Tu We Th Fr Sa"));
}

#[test]
fn three_month_view() {
    // -3 flag shows previous, current, and next month
    let output = cal().args(["-3", "6", "2024"]).output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert!(stdout.contains("May 2024"));
    assert!(stdout.contains("June 2024"));
    assert!(stdout.contains("July 2024"));
}

#[test]
fn three_month_january_wraps() {
    // -3 in January should show December of previous year
    let output = cal().args(["-3", "1", "2024"]).output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert!(stdout.contains("December 2023"));
    assert!(stdout.contains("January 2024"));
    assert!(stdout.contains("February 2024"));
}

#[test]
fn three_month_december_wraps() {
    // -3 in December should show January of next year
    let output = cal().args(["-3", "12", "2024"]).output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert!(stdout.contains("November 2024"));
    assert!(stdout.contains("December 2024"));
    assert!(stdout.contains("January 2025"));
}

#[test]
fn julian_flag() {
    // -j flag should show day-of-year numbering
    let output = cal().args(["-j", "1", "2024"]).output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    // Day 1 of January should be ordinal 1
    assert!(stdout.contains("January 2024"));
    // Last day of January should be ordinal 31
    assert!(
        stdout.contains("31"),
        "January should contain ordinal day 31"
    );
}

#[test]
fn julian_february_ordinals() {
    // In Julian mode, Feb 1 2024 should be day 32
    let output = cal().args(["-j", "2", "2024"]).output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    // Feb 1 = day 32 of year
    assert!(stdout.contains("32"), "Feb 1 should be ordinal day 32");
    // Feb 29 2024 (leap year) = day 60
    assert!(stdout.contains("60"), "Feb 29 should be ordinal day 60");
}

#[test]
fn n_months_flag() {
    // -n 4 should show 4 months starting from the given month
    let output = cal().args(["-n", "4", "3", "2024"]).output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert!(stdout.contains("March 2024"));
    assert!(stdout.contains("April 2024"));
    assert!(stdout.contains("May 2024"));
    assert!(stdout.contains("June 2024"));
}

#[test]
fn month_name_argument() {
    // Month can be specified by name
    cal()
        .args(["jan", "2024"])
        .assert()
        .success()
        .stdout(predicate::str::contains("January 2024"));
}

#[test]
fn month_name_full() {
    cal()
        .args(["january", "2024"])
        .assert()
        .success()
        .stdout(predicate::str::contains("January 2024"));
}

#[test]
fn invalid_month_fails() {
    cal().args(["13", "2024"]).assert().failure();
}

#[test]
fn too_many_args_fails() {
    cal().args(["1", "2", "3", "4"]).assert().failure();
}

#[test]
fn september_1752_renders() {
    // This is the historical Julian/Gregorian gap month. Our implementation
    // uses pure Gregorian, so it should still render without error. We just
    // verify it doesn't crash and contains the header.
    cal()
        .args(["9", "1752"])
        .assert()
        .success()
        .stdout(predicate::str::contains("September 1752"));
}

#[test]
fn day_alignment_march_2024() {
    // March 1 2024 is a Friday
    let output = cal().args(["3", "2024"]).output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    // With Sunday-first, Friday is position 6 (0-indexed 5).
    // The first week should have blanks for Su-Th, then " 1  2"
    let lines: Vec<&str> = stdout.lines().collect();
    let first_week = lines.iter().find(|l| l.trim().starts_with('1')).unwrap();
    // 5 blank day slots (Su Mo Tu We Th) = 15 chars, then " 1  2"
    assert!(
        first_week.contains(" 1  2"),
        "March 2024 first week should have 1 (Fri) and 2 (Sat) together, got: '{first_week}'"
    );
}

#[test]
fn columns_flag() {
    // -c 4 with year view should render 4 columns
    let output = cal().args(["-c", "4", "-y", "2024"]).output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    // With 4 columns, the first row should have Jan, Feb, Mar, Apr
    // Check that January and April appear on the same set of lines
    let lines: Vec<&str> = stdout.lines().collect();
    let has_four_months_row = lines
        .iter()
        .any(|l| l.contains("January") && l.contains("April"));
    assert!(
        has_four_months_row,
        "With 4 columns, January and April should be on the same row"
    );
}
