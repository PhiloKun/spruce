use anyhow::{bail, Context, Result};
use serde_json::Value;

/// Evaluate a JSON dot-notation query against a JSON string
pub fn query_json(content: &str, query: &str, no_color: bool) -> Result<String> {
    let value: Value =
        serde_json::from_str(content).context("Failed to parse JSON")?;

    let result = evaluate_json_query(&value, query)?;
    let formatted = serde_json::to_string_pretty(&result)
        .context("Failed to serialize query result")?;

    Ok(crate::highlight::highlight_json(&formatted, no_color))
}

/// Evaluate a simple XPath-like query against XML
pub fn query_xml(content: &str, query: &str, no_color: bool) -> Result<String> {
    use quick_xml::Reader;

    let raw_parts: Vec<&str> = query
        .trim_start_matches('/')
        .split('/')
        .filter(|p| !p.is_empty())
        .collect();

    // Separate attribute target from element path
    let (parts, attr_target) = if let Some(last) = raw_parts.last() {
        if let Some(at) = last.find('@') {
            let attr_key = &last[at + 1..];
            let mut elements: Vec<&str> = raw_parts[..raw_parts.len() - 1].to_vec();
            if at > 0 {
                elements.push(&last[..at]);
            }
            (elements, Some(attr_key))
        } else {
            (raw_parts.clone(), None)
        }
    } else {
        (raw_parts, None)
    };

    let mut reader = Reader::from_str(content);
    reader.config_mut().trim_text(true);

    let root = build_xml_tree(&mut reader)?;
    let matched_nodes = evaluate_xml_query(&root, &parts);

    if matched_nodes.is_empty() {
        bail!("Query '{}' did not match any nodes", query);
    }

    let output = if let Some(attr_key) = attr_target {
        // Extract attribute values
        let values: Vec<Value> = matched_nodes
            .iter()
            .filter_map(|n| n.find_attr(attr_key))
            .map(|v| Value::String(v.to_string()))
            .collect();
        if values.is_empty() {
            bail!("Attribute '{}' not found on matched nodes", attr_key);
        }
        if values.len() == 1 {
            values.into_iter().next().unwrap()
        } else {
            Value::Array(values)
        }
    } else {
        // Return nodes as JSON
        let json_results: Vec<Value> = matched_nodes.iter().map(|n| xml_node_to_json(n)).collect();
        if json_results.len() == 1 {
            json_results.into_iter().next().unwrap()
        } else {
            Value::Array(json_results)
        }
    };

    let formatted = serde_json::to_string_pretty(&output)
        .context("Failed to serialize query result")?;

    if no_color {
        Ok(formatted)
    } else {
        Ok(crate::highlight::highlight_json(&formatted, no_color))
    }
}

// ---- JSON query engine ----

fn evaluate_json_query(value: &Value, query: &str) -> Result<Value> {
    let query = query.trim();

    if query.is_empty() || query == "." || query == "$" {
        return Ok(value.clone());
    }

    // Strip leading $ or .
    let path = query
        .strip_prefix('$')
        .unwrap_or(query)
        .strip_prefix('.')
        .unwrap_or(query);

    // Handle bracket starts like [0].name
    let path = if !path.starts_with('[') && !path.starts_with('.') {
        path
    } else {
        path.trim_start_matches('.')
    };

    let segments = parse_json_path(path);
    let mut current = value.clone();

    for seg in &segments {
        match seg {
            PathSegment::Key(k) => match current {
                Value::Object(ref m) => {
                    current = m
                        .get(k.as_str())
                        .ok_or_else(|| anyhow::anyhow!("Key '{}' not found", k))?
                        .clone();
                }
                ref other => bail!("Cannot index into {} with key '{}'", current_type(other), k),
            },
            PathSegment::Index(i) => match current {
                Value::Array(ref arr) => {
                    let idx = if *i < 0 {
                        if arr.is_empty() {
                            bail!("Cannot index empty array with {}", i);
                        }
                        arr.len() as isize + i
                    } else {
                        *i
                    };
                    if idx < 0 || idx as usize >= arr.len() {
                        bail!("Index {} out of bounds (length {})", idx, arr.len());
                    }
                    current = arr[idx as usize].clone();
                }
                ref other => bail!("Cannot index into {} with [{}]", current_type(other), i),
            },
            PathSegment::Wildcard => match current {
                Value::Object(ref m) => {
                    let values: Vec<Value> = m.values().cloned().collect();
                    return Ok(Value::Array(values));
                }
                Value::Array(ref arr) => return Ok(Value::Array(arr.clone())),
                ref other => bail!("Cannot use wildcard on {}", current_type(other)),
            },
        }
    }

    Ok(current)
}

#[derive(Debug)]
enum PathSegment {
    Key(String),
    Index(isize),
    Wildcard,
}

fn parse_json_path(path: &str) -> Vec<PathSegment> {
    let mut segments = Vec::new();
    let mut remaining = path;
    let mut current_key = String::new();

    while !remaining.is_empty() {
        if remaining.starts_with('.') {
            if !current_key.is_empty() {
                if current_key == "*" {
                    segments.push(PathSegment::Wildcard);
                } else {
                    segments.push(PathSegment::Key(current_key.clone()));
                }
                current_key.clear();
            }
            remaining = &remaining[1..];
            continue;
        }

        if remaining.starts_with('[') {
            if !current_key.is_empty() {
                if current_key == "*" {
                    segments.push(PathSegment::Wildcard);
                } else {
                    segments.push(PathSegment::Key(current_key.clone()));
                }
                current_key.clear();
            }

            let end = remaining.find(']').unwrap_or(remaining.len());
            let inner = &remaining[1..end];

            if inner == "*" {
                segments.push(PathSegment::Wildcard);
            } else if let Ok(n) = inner.parse::<isize>() {
                segments.push(PathSegment::Index(n));
            } else if !inner.is_empty() {
                // ['key'] syntax
                let key = inner.trim_matches('\'');
                segments.push(PathSegment::Key(key.to_string()));
            }

            remaining = &remaining[if end < remaining.len() { end + 1 } else { remaining.len() }..];
            continue;
        }

        current_key.push(remaining.chars().next().unwrap());
        remaining = &remaining[1..];
    }

    if !current_key.is_empty() {
        if current_key == "*" {
            segments.push(PathSegment::Wildcard);
        } else {
            segments.push(PathSegment::Key(current_key));
        }
    }

    segments
}

fn current_type(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "bool",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

// ---- XML helpers ----

#[derive(Debug, Clone)]
struct XmlNode {
    name: String,
    text: String,
    attrs: std::collections::HashMap<String, String>,
    children: Vec<XmlNode>,
}

impl XmlNode {
    fn new(name: String) -> Self {
        XmlNode {
            name,
            text: String::new(),
            attrs: std::collections::HashMap::new(),
            children: Vec::new(),
        }
    }

    fn find_attr(&self, key: &str) -> Option<&str> {
        self.attrs.get(key).map(|s| s.as_str())
    }
}

/// Build an XML tree from a reader
fn build_xml_tree(reader: &mut quick_xml::Reader<&[u8]>) -> Result<XmlNode> {
    use quick_xml::events::Event;

    let mut root = XmlNode::new("__root__".to_string());
    let mut stack: Vec<XmlNode> = Vec::new();
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                let mut node = XmlNode::new(tag);

                for attr in e.attributes().filter_map(|a| a.ok()) {
                    let k = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                    let v = String::from_utf8_lossy(&attr.value).to_string();
                    node.attrs.insert(k, v);
                }

                stack.push(node);
            }
            Ok(Event::End(ref _e)) => {
                if let Some(node) = stack.pop() {
                    if let Some(parent) = stack.last_mut() {
                        parent.children.push(node);
                    } else {
                        root.children.push(node);
                    }
                }
            }
            Ok(Event::Empty(ref e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                let mut node = XmlNode::new(tag);

                for attr in e.attributes().filter_map(|a| a.ok()) {
                    let k = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                    let v = String::from_utf8_lossy(&attr.value).to_string();
                    node.attrs.insert(k, v);
                }

                if let Some(parent) = stack.last_mut() {
                    parent.children.push(node);
                } else {
                    root.children.push(node);
                }
            }
            Ok(Event::Text(ref e)) => {
                let text = e.unescape().unwrap_or_default().to_string();
                if !text.trim().is_empty() {
                    if let Some(last) = stack.last_mut() {
                        last.text = text.trim().to_string();
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => bail!("XML parse error: {}", e),
            _ => {}
        }
        buf.clear();
    }

    Ok(root)
}

/// Evaluate an XPath-like query by walking the tree.
/// Returns matched nodes (for attribute queries, the attr is extracted in `query_xml`).
fn evaluate_xml_query<'a>(node: &'a XmlNode, parts: &[&str]) -> Vec<&'a XmlNode> {
    if parts.is_empty() {
        return vec![node];
    }

    let mut current: Vec<&XmlNode> = node.children.iter().collect();

    for (depth, part) in parts.iter().enumerate() {
        let (name, attr_req) = parse_xml_part(part);
        let is_last = depth == parts.len() - 1;
        let mut next = Vec::new();

        for child in current {
            if child.name == name || name == "*" || name.is_empty() {
                if is_last {
                    // On last segment: collect node (attr filtered below in query_xml)
                    if let Some(attr_key) = attr_req {
                        if child.find_attr(attr_key).is_some() {
                            next.push(child);
                        }
                    } else {
                        next.push(child);
                    }
                } else {
                    next.extend(&child.children);
                }
            }
        }

        current = next;
        if current.is_empty() {
            return vec![];
        }
    }

    current
}

/// Parse a path segment like "item" or "item@id" into (name, optional_attr_key)
fn parse_xml_part(part: &str) -> (&str, Option<&str>) {
    if let Some(at) = part.find('@') {
        (&part[..at], Some(&part[at + 1..]))
    } else {
        (part, None)
    }
}

fn xml_node_to_json(node: &XmlNode) -> Value {
    if node.children.is_empty() && !node.text.is_empty() {
        if node.attrs.is_empty() {
            return Value::String(node.text.clone());
        }
        let mut m = serde_json::Map::new();
        m.insert("#text".to_string(), Value::String(node.text.clone()));
        for (k, v) in &node.attrs {
            m.insert(format!("@{}", k), Value::String(v.clone()));
        }
        return Value::Object(m);
    }

    if node.children.is_empty() {
        if node.attrs.is_empty() {
            return Value::String(node.name.clone());
        }
        let mut m = serde_json::Map::new();
        for (k, v) in &node.attrs {
            m.insert(format!("@{}", k), Value::String(v.clone()));
        }
        return Value::Object(m);
    }

    let mut m = serde_json::Map::new();
    if !node.text.is_empty() {
        m.insert("#text".to_string(), Value::String(node.text.clone()));
    }
    for (k, v) in &node.attrs {
        m.insert(format!("@{}", k), Value::String(v.clone()));
    }
    for child in &node.children {
        m.insert(child.name.clone(), xml_node_to_json(child));
    }
    Value::Object(m)
}
