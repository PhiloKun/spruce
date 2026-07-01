use colored::Colorize;
use serde_json::Value;

/// Highlight JSON syntax with color
pub fn highlight_json(json: &str, no_color: bool) -> String {
    if no_color {
        return json.to_string();
    }

    let mut out = String::new();
    let mut chars = json.chars().peekable();
    let mut in_string = false;
    let mut in_escape = false;

    while let Some(ch) = chars.next() {
        if in_string {
            if in_escape {
                out.push_str(&ch.to_string().yellow().to_string());
                in_escape = false;
                continue;
            }
            if ch == '\\' {
                in_escape = true;
                out.push_str(&ch.to_string().yellow().to_string());
                continue;
            }
            if ch == '"' {
                in_string = false;
                out.push_str(&ch.to_string().green().to_string());
                continue;
            }
            out.push_str(&ch.to_string().green().to_string());
            continue;
        }

        match ch {
            '"' => {
                in_string = true;
                // Look ahead — if not followed by ':', it's a string value
                let mut ahead = chars.clone().peekable();
                let is_key = ahead.any(|c| c == ':');
                if is_key {
                    out.push_str(&ch.to_string().cyan().to_string());
                } else {
                    out.push_str(&ch.to_string().green().to_string());
                }
            }
            '{' | '}' | '[' | ']' => out.push_str(&ch.to_string().white().bold().to_string()),
            ',' => {
                out.push_str(&ch.to_string().white().to_string());
            }
            ':' => {
                out.push_str(&ch.to_string().white().to_string());
                if let Some(&next) = chars.peek() {
                    if next == ' ' {
                        chars.next();
                        out.push(' ');
                    }
                }
                continue;
            }
            c if c.is_ascii_digit() || c == '-' || c == '.' => {
                let mut num = String::from(c);
                while let Some(&n) = chars.peek() {
                    if n.is_ascii_digit() || n == '.' || n == '-' || n == 'e' || n == 'E' || n == '+'
                    {
                        num.push(n);
                        chars.next();
                    } else {
                        break;
                    }
                }
                out.push_str(&num.magenta().to_string());
                continue;
            }
            't' | 'f' => {
                // true / false
                let mut word = String::from(ch);
                for _ in 0..3 {
                    if let Some(c) = chars.next() {
                        word.push(c);
                    }
                }
                if word == "true" || word == "false" {
                    out.push_str(&word.magenta().to_string());
                } else {
                    out.push_str(&word);
                }
                continue;
            }
            'n' => {
                let mut word = String::from(ch);
                for _ in 0..3 {
                    if let Some(c) = chars.next() {
                        word.push(c);
                    }
                }
                if word == "null" {
                    out.push_str(&word.magenta().to_string());
                } else {
                    out.push_str(&word);
                }
                continue;
            }
            c => out.push(c),
        }
    }

    out
}

/// Highlight XML syntax with color
pub fn highlight_xml(xml: &str, no_color: bool) -> String {
    if no_color {
        return xml.to_string();
    }

    let mut out = String::new();
    let mut chars = xml.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '<' => {
                // Check for closing tag, opening tag, or comment
                if let Some(&next) = chars.peek() {
                    if next == '/' {
                        // Closing tag  </tag>
                        out.push_str(&"<".white().bold().to_string());
                        out.push_str(&"/".white().bold().to_string());
                        chars.next();
                        let mut tag = String::new();
                        while let Some(&c) = chars.peek() {
                            if c == '>' { chars.next(); // consume >
                                break;
                            }
                            tag.push(c);
                            chars.next();
                        }
                        out.push_str(&tag.blue().to_string());
                    } else if next == '?' {
                        // Processing instruction <?xml ... ?>
                        out.push_str(&"<".blue().to_string());
                        let mut pi = String::new();
                        while let Some(&c) = chars.peek() {
                            pi.push(c);
                            chars.next();
                            if c == '?' {
                                if let Some(&'>') = chars.peek() {
                                    pi.push('>');
                                    chars.next();
                                    break;
                                }
                            }
                        }
                        out.push_str(&pi.blue().to_string());
                        continue;
                    } else if next == '!' {
                        // Comment <!-- ... -->
                        let mut comment = String::from("<!");
                        chars.next();
                        while let Some(&c) = chars.peek() {
                            comment.push(c);
                            chars.next();
                            if c == '-' {
                                if let Some(&'-') = chars.peek() {
                                    comment.push('-');
                                    chars.next();
                                    if let Some(&'>') = chars.peek() {
                                        comment.push('>');
                                        chars.next();
                                        break;
                                    }
                                }
                            }
                        }
                        out.push_str(&comment.bright_black().to_string());
                        continue;
                    } else {
                        // Opening tag  <tag ...>
                        out.push_str(&"<".white().bold().to_string());
                        let mut tag = String::new();
                        let mut in_attr = false;

                        while let Some(&c) = chars.peek() {
                            if c == '>' || (c == '/' && chars.clone().nth(1) == Some('>')) {
                                break;
                            }
                            if c == ' ' && !tag.is_empty() {
                                in_attr = true;
                            }
                            let c = chars.next().unwrap();

                            if in_attr {
                                if c == '=' {
                                    out.push_str(&tag.blue().to_string());
                                    tag.clear();
                                    out.push_str(&"=".white().to_string());
                                } else if c == '"' {
                                    out.push_str(&"\"".yellow().to_string());
                                    let mut val = String::new();
                                    while let Some(&vc) = chars.peek() {
                                        if vc == '"' {
                                            break;
                                        }
                                        val.push(vc);
                                        chars.next();
                                    }
                                    out.push_str(&val.yellow().to_string());
                                    // consume the closing quote
                                    if let Some(&'"') = chars.peek() {
                                        chars.next();
                                    }
                                    out.push_str(&"\"".yellow().to_string());
                                } else {
                                    tag.push(c);
                                }
                            } else {
                                tag.push(c);
                            }
                        }

                        if !tag.is_empty() {
                            out.push_str(&tag.blue().to_string());
                        }

                        if let Some(&'/') = chars.peek() {
                            out.push_str(&"/".white().bold().to_string());
                            chars.next();
                        }
                        // consume the closing > and push it here
                        if let Some(&next_ch) = chars.peek() {
                            if next_ch == '>' {
                                chars.next();
                            }
                        }
                        out.push_str(&">".white().bold().to_string());
                        continue; // skip the fallthrough push
                    }
                }
                out.push_str(&">".white().bold().to_string());
            }
            '>' => {
                out.push_str(&">".white().bold().to_string());
            }
            c => out.push(c),
        }
    }

    out
}

/// Format a JSON value as a human-readable tree
#[allow(dead_code)]
pub fn format_json_tree(value: &Value, prefix: &str, is_last: bool, no_color: bool) -> String {
    let _connector = if is_last { "└── " } else { "├── " };
    let _child_prefix = if is_last { "    " } else { "│   " };
    let mut out = String::new();

    if prefix.is_empty() {
        // Root
        match value {
            Value::Object(m) => {
                let keys: Vec<&String> = m.keys().collect();
                out.push_str("{\n");
                for (i, key) in keys.iter().enumerate() {
                    let last = i == keys.len() - 1;
                    let c = if last { "└── " } else { "├── " };
                    let p = if last { "    " } else { "│   " };
                    let k = if no_color {
                        format!("{}", key)
                    } else {
                        key.cyan().to_string()
                    };
                    out.push_str(&format!("{}{}: ", c, k));
                    out.push_str(&format_json_tree(
                        &m[*key],
                        &p,
                        last,
                        no_color,
                    ));
                }
                if !keys.is_empty() {
                    out.push_str(prefix);
                }
                out.push_str("}\n");
            }
            Value::Array(arr) => {
                out.push_str("[\n");
                for (i, item) in arr.iter().enumerate() {
                    let last = i == arr.len() - 1;
                    let c = if last { "└── " } else { "├── " };
                    let p = if last { "    " } else { "│   " };
                    let idx = if no_color {
                        format!("[{}]", i)
                    } else {
                        format!("[{}]", i).white().bold().to_string()
                    };
                    out.push_str(&format!("{}{} ", prefix, c));
                    out.push_str(&idx);
                    out.push(' ');
                    out.push_str(&format_json_tree(item, &format!("{}{}", prefix, p), last, no_color));
                }
                if !arr.is_empty() {
                    out.push_str(prefix);
                }
                out.push_str("]\n");
            }
            Value::String(s) => {
                if no_color {
                    out.push_str(&format!("\"{}\"", s));
                } else {
                    out.push_str(&format!("\"{}\"", s).green().to_string());
                }
            }
            Value::Number(n) => {
                if no_color {
                    out.push_str(&format!("{}", n));
                } else {
                    out.push_str(&format!("{}", n).magenta().to_string());
                }
            }
            Value::Bool(b) => {
                if no_color {
                    out.push_str(&format!("{}", b));
                } else {
                    out.push_str(&format!("{}", b).magenta().to_string());
                }
            }
            Value::Null => {
                if no_color {
                    out.push_str("null");
                } else {
                    out.push_str(&"null".magenta().to_string());
                }
            }
        }
        return out;
    }

    // Non-root
    match value {
        Value::Object(m) => {
            out.push_str("{\n");
            let keys: Vec<&String> = m.keys().collect();
            for (i, key) in keys.iter().enumerate() {
                let last = i == keys.len() - 1;
                let c = if last { "└── " } else { "├── " };
                let p = if last { "    " } else { "│   " };
                let k = if no_color {
                    format!("{}", key)
                } else {
                    key.cyan().to_string()
                };
                out.push_str(&format!("{}{}{}: ", prefix, c, k));
                out.push_str(&format_json_tree(
                    &m[*key],
                    &format!("{}{}", prefix, p),
                    last,
                    no_color,
                ));
            }
            if !keys.is_empty() {
                out.push_str(prefix);
            }
            out.push_str("}\n");
        }
        Value::Array(arr) => {
            out.push_str("[\n");
            for (i, item) in arr.iter().enumerate() {
                let last = i == arr.len() - 1;
                let c = if last { "└── " } else { "├── " };
                let p = if last { "    " } else { "│   " };
                let idx = if no_color {
                    format!("[{}]", i)
                } else {
                    format!("[{}]", i).white().bold().to_string()
                };
                out.push_str(&format!("{}{}{} ", prefix, c, idx));
                out.push_str(&format_json_tree(
                    item,
                    &format!("{}{}", prefix, p),
                    last,
                    no_color,
                ));
            }
            if !arr.is_empty() {
                out.push_str(prefix);
            }
            out.push_str("]\n");
        }
        Value::String(s) => {
            if no_color {
                out.push_str(&format!("\"{}\"", s));
            } else {
                out.push_str(&format!("\"{}\"", s).green().to_string());
            }
            out.push('\n');
        }
        Value::Number(n) => {
            if no_color {
                out.push_str(&format!("{}", n));
            } else {
                out.push_str(&format!("{}", n).magenta().to_string());
            }
            out.push('\n');
        }
        Value::Bool(b) => {
            if no_color {
                out.push_str(&format!("{}", b));
            } else {
                out.push_str(&format!("{}", b).magenta().to_string());
            }
            out.push('\n');
        }
        Value::Null => {
            if no_color {
                out.push_str("null");
            } else {
                out.push_str(&"null".magenta().to_string());
            }
            out.push('\n');
        }
    }

    out
}

/// Format XML tree view
pub fn format_xml_tree(xml: &str, no_color: bool) -> Result<String, anyhow::Error> {
    use quick_xml::events::Event;
    use quick_xml::Reader;

    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut out = String::new();
    let mut depth: Vec<(usize, String)> = Vec::new();
    let mut indent_level: usize = 0;

    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                let attrs: Vec<String> = e
                    .attributes()
                    .filter_map(|a| a.ok())
                    .map(|a| {
                        let k = String::from_utf8_lossy(a.key.as_ref()).to_string();
                        let v = String::from_utf8_lossy(&a.value).to_string();
                        if no_color {
                            format!("@{}=\"{}\"", k, v)
                        } else {
                            format!(
                                "{}={}",
                                k.yellow().to_string(),
                                format!("\"{}\"", v).yellow().to_string()
                            )
                        }
                    })
                    .collect();

                let prefix = "│   ".repeat(indent_level.saturating_sub(1));

                let tag_display = if no_color {
                    tag.clone()
                } else {
                    tag.blue().to_string()
                };

                if attrs.is_empty() {
                    out.push_str(&format!("{}├── {}\n", prefix, tag_display));
                } else {
                    out.push_str(&format!(
                        "{}├── {} [{}]\n",
                        prefix,
                        tag_display,
                        attrs.join(", ")
                    ));
                }

                depth.push((indent_level, tag.clone()));
                indent_level += 1;
            }
            Ok(Event::Empty(ref e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                let attrs: Vec<String> = e
                    .attributes()
                    .filter_map(|a| a.ok())
                    .map(|a| {
                        let k = String::from_utf8_lossy(a.key.as_ref()).to_string();
                        let v = String::from_utf8_lossy(&a.value).to_string();
                        if no_color {
                            format!("@{}=\"{}\"", k, v)
                        } else {
                            format!(
                                "{}={}",
                                k.yellow().to_string(),
                                format!("\"{}\"", v).yellow().to_string()
                            )
                        }
                    })
                    .collect();

                let prefix = "│   ".repeat(indent_level.saturating_sub(1));

                let tag_display = if no_color {
                    tag.clone()
                } else {
                    tag.blue().to_string()
                };

                if attrs.is_empty() {
                    out.push_str(&format!("{}├── {} (empty)\n", prefix, tag_display));
                } else {
                    out.push_str(&format!(
                        "{}├── {} [{}] (empty)\n",
                        prefix,
                        tag_display,
                        attrs.join(", ")
                    ));
                }
            }
            Ok(Event::Text(ref e)) => {
                let text = e.unescape().unwrap_or_default().to_string();
                if !text.trim().is_empty() {
                    let prefix = "│   ".repeat(indent_level.saturating_sub(1));
                    let text_display = if no_color {
                        format!("\"{}\"", text.trim())
                    } else {
                        format!("\"{}\"", text.trim()).green().to_string()
                    };
                    out.push_str(&format!("{}├── {}\n", prefix, text_display));
                }
            }
            Ok(Event::End(ref _e)) => {
                if indent_level > 0 {
                    indent_level -= 1;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                return Err(anyhow::anyhow!("XML parse error: {}", e));
            }
            _ => {}
        }
    }

    Ok(out)
}
