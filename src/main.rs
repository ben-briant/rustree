pub mod rustree;

use clap::Parser;
use rustree::{traverse_layer, OptionsBuilder};
use std::{io::Result, path::Path};

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Do not colorize the output
    #[arg(short, long, action)]
    no_colors: bool,

    /// Limit the depth of the file tree to traverse
    #[arg(short, long)]
    depth: Option<usize>,

    /// Show all files, including hidden files
    #[arg(short, long, action)]
    show_hidden: bool,

    /// The root path
    path: Option<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();
    if args.no_colors {
        colored::control::set_override(false);
    }
    let mut options_builder = OptionsBuilder::new();
    if let Some(depth) = args.depth {
        options_builder.with_depth(depth);
    }
    if args.show_hidden {
        options_builder.show_hidden();
    }
    let options = options_builder.build();
    let depth = 0;
    let path = args.path.unwrap_or_else(|| ".".to_string());
    traverse_layer(Path::new(&path), depth, "".to_string(), options)?;
    Ok(())
}
