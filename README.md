
<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="https://img.shields.io/badge/🌲-Spruce-2ea043?style=for-the-badge">
    <img alt="Spruce" src="https://img.shields.io/badge/🌲-Spruce-2ea043?style=for-the-badge" width="200">
  </picture>
  <br>
  <i>Make your JSON & XML tidy and beautiful</i>
</p>

<p align="center">
  <a href="https://github.com/PhiloKun/spruce/blob/main/LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="MIT License"></a>
  <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/rust-1.75+-orange.svg" alt="Rust 1.75+"></a>
  <img src="https://img.shields.io/badge/platform-macOS|Linux-lightgrey" alt="Platform">
  <img src="https://img.shields.io/github/v/release/PhiloKun/spruce?include_prereleases&label=version" alt="Version">
</p>

<p align="center">
  <b>format</b> &nbsp;•&nbsp; <b>query</b> &nbsp;•&nbsp; <b>tree</b> &nbsp;•&nbsp; <b>color</b>
</p>

---

**Spruce** is a command-line tool to **format**, **query**, and **explore** JSON and XML data — with beautiful syntax highlighting and tree views. Like `jq` for JSON, but also speaks XML, and looks good doing it.

## ✨ Features

- **📝 Pretty-print** — Beautify JSON & XML with configurable indentation
- **🔍 Query** — Navigate JSON with dot-notation (`.users[0].name`) or XML with path expressions (`/root/item/@id`)
- **🌳 Tree view** — See your data structure at a glance
- **🎨 Syntax highlighting** — Colorized output for keys, strings, numbers, booleans, nulls (JSON) and tags, attributes, values (XML)
- **📂 File & pipe** — Read from files or stdin, write to stdout or files
- **🧠 Auto-detect** — Automatically detects JSON vs XML by content or file extension
- **🪶 Fast & small** — Written in Rust, single binary, no runtime dependencies

## 🚀 Installation

### From source (recommended)

```bash
cargo install --git https://github.com/PhiloKun/spruce
```

### From crates.io (once published)

```bash
cargo install spruce
```

### Build from source

```bash
git clone https://github.com/PhiloKun/spruce.git
cd spruce
cargo build --release
# Binary at ./target/release/spruce
```

## 📖 Usage

### Format

```bash
# JSON from stdin
echo '{"name":"Spruce","version":"0.1.0"}' | spruce format

# XML from file
spruce fmt data.xml

# With custom indentation
spruce fmt data.json --indent 4

# Write to file
spruce fmt data.json -o formatted.json

# Force type
cat data | spruce fmt --type xml

# No color
spruce fmt data.json --no-color
```

### Query

```bash
# Simple field access (JSON)
echo '{"user":{"name":"Alice"}}' | spruce query '.user.name'

# Array index
spruce q '.users[0].email' data.json

# Wildcard
spruce q '.items.*.id' data.json

# XML path
spruce q '/root/person' data.xml

# XML attribute query
spruce q '/root/person/@id' data.xml

# Output to file
spruce q '.users' data.json -o users.json
```

### Tree view

```bash
# JSON tree
spruce tree data.json

# XML tree
spruce tree data.xml --type xml

# Alias
spruce t data.json
```

### Pipe chaining

```bash
# Fetch -> Format
curl -s https://api.example.com/data | spruce fmt

# Format -> Query
cat data.json | spruce fmt | spruce q '.name'

# Chain queries
cat data.json | spruce q '.users' | spruce q '.[0].email'
```

## 🎨 Color Scheme

| Token | JSON | XML |
|-------|------|-----|
| **Keys / Tags** | Cyan | Blue |
| **Strings** | Green | Green |
| **Numbers** | Magenta | — |
| **Booleans / Null** | Magenta | — |
| **Attributes** | — | Yellow |
| **Brackets / Delimiters** | White (bold) | White (bold) |
| **Comments** | — | Bright black |

## 💡 Tips

- Use `spruce format --help` to see all options for the format command
- Pipe into `spruce query` to extract specific fields after formatting
- Use `--no-color` when redirecting output to a file or using in scripts
- The `fmt`, `q`, and `t` aliases save keystrokes for interactive use

## 🤝 Contributing

Contributions are welcome! Feel free to open an issue or submit a PR.

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing`)
3. Commit your changes (`git commit -am 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing`)
5. Open a Pull Request

## 📄 License

MIT © [PhiloKun](https://github.com/PhiloKun)

---

<p align="center">
  🌲 <i>Spruce up your data.</i>
</p>
