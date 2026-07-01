
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

| [English](#-introduction) | [中文](#-简介) |
|--------------------------|---------------|

---

## 🌟 Introduction

**Spruce** is a command-line tool to **format**, **query**, and **explore** JSON and XML data — with beautiful syntax highlighting and tree views. Like `jq` for JSON, but also speaks XML, and looks good doing it.

---

## 🌟 简介

**Spruce** 是一个命令行工具，用于**格式化**、**查询**和**浏览** JSON 和 XML 数据——内置漂亮的语法高亮和树形视图。就像 JSON 界的 `jq`，但它也支持 XML，而且颜值在线。

---

## ✨ Features / 功能特性

| English | 中文 |
|---------|------|
| **📝 Pretty-print** — Beautify JSON & XML with configurable indentation | **📝 格式化** — 美化 JSON 和 XML，缩进可调 |
| **🔍 Query** — Dot-notation for JSON (`.users[0].name`), path expressions for XML (`/root/item/@id`) | **🔍 查询** — JSON 点号路径与 XML 路径表达式 |
| **🌳 Tree view** — See your data structure at a glance | **🌳 树形视图** — 数据结构一目了然 |
| **🎨 Syntax highlighting** — Colorized output for keys, strings, numbers, tags, attributes | **🎨 语法高亮** — 键名、字符串、数字、标签、属性彩色输出 |
| **📂 File & pipe** — Read from files or stdin, write to stdout or files | **📂 文件与管道** — 支持文件读写和标准输入/输出 |
| **🧠 Auto-detect** — Automatically detects JSON vs XML | **🧠 自动识别** — 自动判断 JSON 或 XML 格式 |
| **🪶 Fast & small** — Written in Rust, single binary | **🪶 快速轻量** — Rust 编写，单个可执行文件 |

---

## 🚀 Installation / 安装

### Via Cargo / 通过 Cargo 安装

```bash
# From GitHub (recommended / 推荐)
cargo install --git https://github.com/PhiloKun/spruce

# From source / 源码编译
git clone https://github.com/PhiloKun/spruce.git
cd spruce
cargo build --release
# Binary at ./target/release/spruce
```

### Via Homebrew (coming soon / 即将支持)

```bash
# brew install spruce
```

---

## 📖 Usage / 使用指南

### Format / 格式化

```bash
# JSON from stdin / 从标准输入读取 JSON
echo '{"name":"Spruce","version":"0.1.0"}' | spruce format

# XML from file / 从文件读取 XML
spruce fmt data.xml

# Custom indent / 自定义缩进
spruce fmt data.json --indent 4

# Write to file / 输出到文件
spruce fmt data.json -o formatted.json

# Force type / 强制指定格式类型
cat data | spruce fmt --type xml

# Disable color / 禁用颜色
spruce fmt data.json --no-color
```

### Query / 查询

```bash
# JSON field access / JSON 字段访问
echo '{"user":{"name":"Alice"}}' | spruce query '.user.name'

# Array index / 数组索引
spruce q '.users[0].email' data.json

# Wildcard / 通配符
spruce q '.items.*.id' data.json

# XML path / XML 路径
spruce q '/root/person' data.xml

# XML attribute query / XML 属性查询
spruce q '/root/person/@id' data.xml

# Output to file / 输出到文件
spruce q '.users' data.json -o users.json
```

### Tree view / 树形视图

```bash
# JSON tree / JSON 树
spruce tree data.json

# XML tree / XML 树
spruce tree data.xml --type xml

# Alias / 别名
spruce t data.json
```

### Pipe chaining / 管道链

```bash
# Fetch -> Format / 获取 -> 格式化
curl -s https://api.example.com/data | spruce fmt

# Format -> Query / 格式化 -> 查询
cat data.json | spruce fmt | spruce q '.name'

# Chain queries / 查询链
cat data.json | spruce q '.users' | spruce q '.[0].email'
```

---

## 🎨 Color Scheme / 色彩方案

| Token | JSON | XML |
|-------|------|-----|
| **Keys / Tags** | Cyan / 青色 | Blue / 蓝色 |
| **Strings** | Green / 绿色 | Green / 绿色 |
| **Numbers** | Magenta / 品红 | — |
| **Booleans / Null** | Magenta / 品红 | — |
| **Attributes** | — | Yellow / 黄色 |
| **Brackets / Delimiters** | White (bold) | White (bold) |
| **Comments** | — | Bright black / 亮黑 |

---

## 💡 Tips / 小贴士

- `spruce format --help` — View all format options / 查看全部格式化选项
- Pipe into `spruce query` to extract fields after formatting / 格式化后通过管道传入 `spruce query` 提取字段
- Use `--no-color` when redirecting to a file / 重定向到文件时使用 `--no-color`
- Aliases `fmt`, `q`, `t` save keystrokes / 别名 `fmt`、`q`、`t` 节省按键

---

## 🤝 Contributing / 贡献指南

Contributions are welcome! / 欢迎贡献代码！

1. Fork the repository / Fork 本仓库
2. Create a feature branch (`git checkout -b feature/amazing`)
3. Commit your changes (`git commit -am 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing`)
5. Open a Pull Request

---

## 📄 License / 许可证

MIT © [PhiloKun](https://github.com/PhiloKun)

---

<p align="center">
  🌲 <i>Spruce up your data. / 让数据整洁起来。</i>
</p>
