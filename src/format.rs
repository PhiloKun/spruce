use crate::highlight;
use anyhow::{bail, Context, Result};
use std::io::Read;

/// Auto-detect format type from file extension or content sniffing
pub fn detect_format(content: &str, file_path: Option<&str>, hint: Option<&str>) -> Result<&'static str> {
    // 1. Explicit hint wins
    if let Some(t) = hint {
        match t.to_lowercase().as_str() {
            "json" => return Ok("json"),
            "xml" => return Ok("xml"),
            other => bail!("Unknown format type '{}'. Use 'json' or 'xml'.", other),
        }
    }

    // 2. File extension
    if let Some(path) = file_path {
        let lower = path.to_lowercase();
        if lower.ends_with(".json") {
            return Ok("json");
        }
        if lower.ends_with(".xml") {
            return Ok("xml");
        }
    }

    // 3. Content sniffing
    let trimmed = content.trim_start();
    if trimmed.starts_with('{') || trimmed.starts_with('[') {
        return Ok("json");
    }
    if trimmed.starts_with('<') {
        return Ok("xml");
    }

    bail!(
        "Unable to detect format. Use --type json or --type xml to specify.\n\
         Content starts with: {:?}",
        trimmed.chars().take(20).collect::<String>()
    )
}

/// Read input from file or stdin
pub fn read_input(file: Option<&str>) -> Result<String> {
    match file {
        Some(path) => {
            let content = std::fs::read_to_string(path)
                .with_context(|| format!("Failed to read file: {}", path))?;
            Ok(content)
        }
        None => {
            let mut buf = String::new();
            std::io::stdin()
                .read_to_string(&mut buf)
                .context("Failed to read from stdin")?;
            if buf.is_empty() {
                bail!("No input provided. Pipe data or specify a file.");
            }
            Ok(buf)
        }
    }
}

/// Write output to file or stdout
pub fn write_output(content: &str, file: Option<&str>) -> Result<()> {
    match file {
        Some(path) => {
            std::fs::write(path, content)
                .with_context(|| format!("Failed to write to file: {}", path))?;
            eprintln!("{} Written to {}", "✔".green(), path);
        }
        None => {
            print!("{}", content);
        }
    }
    Ok(())
}

/// Format JSON with optional syntax highlighting
pub fn format_json(content: &str, indent: usize, no_color: bool) -> Result<String> {
    let value: serde_json::Value =
        serde_json::from_str(content).context("Failed to parse JSON")?;

    let formatted = if indent == 0 {
        serde_json::to_string(&value).context("Failed to serialize JSON")?
    } else {
        serde_json::to_string_pretty(&value).context("Failed to serialize JSON")?
    };

    // Apply custom indent if not default 2
    let formatted = if indent != 2 {
        let mut result = String::new();
        for line in formatted.lines() {
            let trimmed = line.trim_start();
            let leading = line.len() - trimmed.len();
            let new_leading = " ".repeat(leading / 2 * indent);
            result.push_str(&new_leading);
            result.push_str(trimmed);
            result.push('\n');
        }
        result.trim_end().to_string()
    } else {
        formatted
    };

    Ok(highlight::highlight_json(&formatted, no_color))
}

/// Format XML with optional syntax highlighting
pub fn format_xml(content: &str, indent: usize, no_color: bool) -> Result<String> {
    // Parse and re-serialize XML for pretty printing
    let mut reader = quick_xml::Reader::from_str(content);
    reader.config_mut().trim_text(true);

    let mut writer = quick_xml::Writer::new_with_indent(Vec::new(), b' ', indent);

    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(quick_xml::events::Event::Start(ref e)) => {
                writer
                    .write_event(quick_xml::events::Event::Start(e.clone()))
                    .context("Failed to write XML start element")?;
            }
            Ok(quick_xml::events::Event::End(ref e)) => {
                writer
                    .write_event(quick_xml::events::Event::End(e.clone()))
                    .context("Failed to write XML end element")?;
            }
            Ok(quick_xml::events::Event::Empty(ref e)) => {
                writer
                    .write_event(quick_xml::events::Event::Empty(e.clone()))
                    .context("Failed to write XML empty element")?;
            }
            Ok(quick_xml::events::Event::Text(ref e)) => {
                writer
                    .write_event(quick_xml::events::Event::Text(e.clone()))
                    .context("Failed to write XML text")?;
            }
            Ok(quick_xml::events::Event::CData(ref e)) => {
                writer
                    .write_event(quick_xml::events::Event::CData(e.clone()))
                    .context("Failed to write XML CDATA")?;
            }
            Ok(quick_xml::events::Event::Comment(ref e)) => {
                writer
                    .write_event(quick_xml::events::Event::Comment(e.clone()))
                    .context("Failed to write XML comment")?;
            }
            Ok(quick_xml::events::Event::PI(ref e)) => {
                writer
                    .write_event(quick_xml::events::Event::PI(e.clone()))
                    .context("Failed to write XML PI")?;
            }
            Ok(quick_xml::events::Event::DocType(ref e)) => {
                writer
                    .write_event(quick_xml::events::Event::DocType(e.clone()))
                    .context("Failed to write XML DocType")?;
            }
            Ok(quick_xml::events::Event::Eof) => break,
            Err(e) => bail!("XML parse error: {}", e),
            _ => {}
        }
        buf.clear();
    }

    let bytes = writer.into_inner();
    let formatted = String::from_utf8(bytes).context("XML output is not valid UTF-8")?;

    if no_color {
        Ok(formatted)
    } else {
        Ok(highlight::highlight_xml(&formatted, false))
    }
}

/// Generate a tree view for JSON
pub fn tree_json(content: &str, no_color: bool) -> Result<String> {
    let value: serde_json::Value =
        serde_json::from_str(content).context("Failed to parse JSON")?;
    Ok(highlight::format_json_tree(&value, "", false, no_color))
}

/// Generate a tree view for XML
pub fn tree_xml(content: &str, no_color: bool) -> Result<String> {
    highlight::format_xml_tree(content, no_color)
}

// Color helper for terminal output
use colored::Colorize;
