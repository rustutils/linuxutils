use clap_mangen::Man;
use linuxutils_common::man::ManContent;
use std::{fs, io::Write, process::ExitCode};

pub fn run(output: &str) -> ExitCode {
    if let Err(e) = fs::create_dir_all(output) {
        eprintln!("xtask mangen: failed to create output directory: {e}");
        return ExitCode::FAILURE;
    }

    for tool in linuxutils::TOOLS {
        let cmd = (tool.command)();
        let name = cmd.get_name().to_string();
        let path = format!("{output}/{name}.1");
        let mut f = match fs::File::create(&path) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("xtask mangen: failed to create {path}: {e}");
                return ExitCode::FAILURE;
            }
        };
        if let Err(e) = render_man(&mut f, cmd, tool.man) {
            eprintln!("xtask mangen: failed to render {name}: {e}");
            return ExitCode::FAILURE;
        }
        println!("{path}");
    }

    ExitCode::SUCCESS
}

/// Compose a man page by interleaving clap_mangen's auto-rendered
/// sections with the tool's hand-written extras.
fn render_man(
    out: &mut impl Write,
    cmd: clap::Command,
    extras: &ManContent,
) -> std::io::Result<()> {
    // clap_mangen panics on render_version_section / render_authors_section
    // when these aren't set; mirror Man::render's internal gating.
    let has_version =
        cmd.get_long_version().is_some() || cmd.get_version().is_some();
    let has_author = cmd.get_author().is_some();

    let man = Man::new(cmd);

    man.render_title(out)?;
    man.render_name_section(out)?;
    man.render_synopsis_section(out)?;

    // DESCRIPTION: clap_mangen would emit a short blurb from clap's
    // `about`. Override with our richer prose if provided, fall back to
    // clap's auto version otherwise.
    if let Some(desc) = extras.description {
        writeln!(out, ".SH DESCRIPTION")?;
        writeln!(out, "{desc}")?;
    } else {
        man.render_description_section(out)?;
    }

    man.render_options_section(out)?;

    for (title, body) in extras.extra_sections {
        writeln!(out, ".SH {title}")?;
        writeln!(out, "{body}")?;
    }

    if has_version {
        man.render_version_section(out)?;
    }
    if has_author {
        man.render_authors_section(out)?;
    }
    Ok(())
}
