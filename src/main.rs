mod cli;

use clap::Parser;
use colored::Colorize;
use spruce::{format, query};
use std::process;

fn main() {
    let args = cli::Cli::parse();

    let result = match &args.command {
        cli::Commands::Format {
            file,
            indent,
            output,
            format_type,
            no_color,
        } => cmd_format(file.as_deref(), *indent, output.as_deref(), format_type.as_deref(), *no_color),
        cli::Commands::Query {
            query,
            file,
            output,
            format_type,
            no_color,
        } => cmd_query(query, file.as_deref(), output.as_deref(), format_type.as_deref(), *no_color),
        cli::Commands::Tree {
            file,
            format_type,
            no_color,
        } => cmd_tree(file.as_deref(), format_type.as_deref(), *no_color),
    };

    if let Err(e) = result {
        eprintln!(
            "{} {}",
            "✖".red().bold(),
            format!("{}", e).red()
        );
        process::exit(1);
    }
}

fn cmd_format(
    file: Option<&str>,
    indent: usize,
    output: Option<&str>,
    format_type: Option<&str>,
    no_color: bool,
) -> anyhow::Result<()> {
    let content = format::read_input(file)?;
    let fmt = format::detect_format(&content, file, format_type)?;

    let formatted = match fmt {
        "json" => format::format_json(&content, indent, no_color)?,
        "xml" => format::format_xml(&content, indent, no_color)?,
        _ => unreachable!(),
    };

    format::write_output(&formatted, output)?;
    Ok(())
}

fn cmd_query(
    query: &str,
    file: Option<&str>,
    output: Option<&str>,
    format_type: Option<&str>,
    no_color: bool,
) -> anyhow::Result<()> {
    let content = format::read_input(file)?;
    let fmt = format::detect_format(&content, file, format_type)?;

    let result = match fmt {
        "json" => query::query_json(&content, query, no_color)?,
        "xml" => query::query_xml(&content, query, no_color)?,
        _ => unreachable!(),
    };

    format::write_output(&result, output)?;
    Ok(())
}

fn cmd_tree(
    file: Option<&str>,
    format_type: Option<&str>,
    no_color: bool,
) -> anyhow::Result<()> {
    let content = format::read_input(file)?;
    let fmt = format::detect_format(&content, file, format_type)?;

    let tree = match fmt {
        "json" => format::tree_json(&content, no_color)?,
        "xml" => format::tree_xml(&content, no_color)?,
        _ => unreachable!(),
    };

    println!("{}", tree);
    Ok(())
}
