use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "spruce", version)]
#[command(about = "🌲 Spruce — make your JSON & XML tidy and beautiful")]
#[command(long_about = "Spruce formats, queries, and displays JSON and XML data\nwith syntax highlighting, tree views, and flexible input/output.")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Pretty-print JSON or XML with syntax highlighting
    #[command(visible_alias = "fmt")]
    Format {
        /// Input file (reads from stdin if not provided)
        file: Option<String>,

        /// Number of spaces per indent level
        #[arg(short, long, default_value_t = 2)]
        indent: usize,

        /// Write output to file instead of stdout
        #[arg(short, long)]
        output: Option<String>,

        /// Force format type: json | xml
        #[arg(short = 't', long = "type", value_name = "TYPE")]
        format_type: Option<String>,

        /// Disable colored output
        #[arg(long)]
        no_color: bool,
    },

    /// Query data using JSON dot-notation or XPath-like paths
    #[command(visible_alias = "q")]
    Query {
        /// Query expression
        /// JSON examples:  .name  .items[0].id  .users.*.email
        /// XML  examples:  /root/item  /root/item/@id  /root/item/text()
        query: String,

        /// Input file (reads from stdin if not provided)
        file: Option<String>,

        /// Write output to file instead of stdout
        #[arg(short, long)]
        output: Option<String>,

        /// Force format type: json | xml
        #[arg(short = 't', long = "type", value_name = "TYPE")]
        format_type: Option<String>,

        /// Disable colored output
        #[arg(long)]
        no_color: bool,
    },

    /// Display data structure as an interactive tree
    #[command(visible_alias = "t")]
    Tree {
        /// Input file (reads from stdin if not provided)
        file: Option<String>,

        /// Force format type: json | xml
        #[arg(short = 't', long = "type", value_name = "TYPE")]
        format_type: Option<String>,

        /// Disable colored output
        #[arg(long)]
        no_color: bool,
    },
}
