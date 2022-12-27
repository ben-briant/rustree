use std::fs::read_dir;
use std::io::Result;
use std::os::unix::prelude::PermissionsExt;
use std::path::Path;
use std::{fmt::Display, fs::FileType, path::PathBuf};

use colored::Colorize;

#[derive(Clone, Copy)]
pub struct Options {
    pub depth: Option<usize>,
    pub show_hidden: bool,
}

pub struct OptionsBuilder {
    inner: Options,
}

impl OptionsBuilder {
    pub fn new() -> Self {
        Self {
            inner: Options {
                depth: None,
                show_hidden: false,
            },
        }
    }

    pub fn build(self) -> Options {
        self.inner
    }

    pub fn with_depth(&mut self, depth: usize) {
        self.inner.depth = Some(depth);
    }

    pub fn show_hidden(&mut self) {
        self.inner.show_hidden = true;
    }
}

impl Default for OptionsBuilder {
    fn default() -> Self {
        Self::new()
    }
}

struct File {
    pub name: String,
    pub file_type: MyFileType,
    pub path: PathBuf,
    pub is_executable: bool,
}

impl Display for File {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let colored = match (&self.file_type, self.is_executable) {
            (MyFileType::File, false) => self.name.white(),
            (MyFileType::File, true) => self.name.green(),
            (MyFileType::Dir, _) => self.name.blue(),
            (MyFileType::Symlink, _) => self.name.cyan(),
        };
        write!(f, "{colored}")
    }
}

enum MyFileType {
    File,
    Dir,
    Symlink,
}

impl From<FileType> for MyFileType {
    fn from(f: FileType) -> Self {
        if f.is_file() {
            Self::File
        } else if f.is_dir() {
            Self::Dir
        } else if f.is_symlink() {
            Self::Symlink
        } else {
            unimplemented!("Haven't handled other file types yet")
        }
    }
}

pub fn traverse_layer(path: &Path, depth: usize, prefix: String, options: Options) -> Result<()> {
    if let Some(max_depth) = options.depth {
        if depth >= max_depth {
            return Ok(());
        }
    }
    let mut file_it = read_dir(path)?
        .filter_map(|f| {
            let f = f.ok()?;
            let f_name = f.file_name().to_str()?.to_string();
            let f_type = MyFileType::from(f.file_type().ok()?);
            Some(File {
                name: f_name,
                file_type: f_type,
                path: f.path(),
                is_executable: f.metadata().ok()?.permissions().mode() & 0o111 != 0,
            })
        })
        .filter(|f| {
            if options.show_hidden {
                true
            } else {
                !f.name.starts_with('.')
            }
        })
        .peekable();
    while let Some(f) = file_it.next() {
        print_current_item(&f, &prefix, file_it.peek().is_none());
        if let MyFileType::Dir = f.file_type {
            traverse_layer(
                &f.path,
                depth + 1,
                prefix.to_string()
                    + if file_it.peek().is_some() {
                        "│   "
                    } else {
                        "    "
                    },
                options,
            )
            .unwrap();
        }
    }

    Ok(())
}

fn print_current_item(f: &File, prefix: &str, is_last: bool) {
    println!("{}{}{}", prefix, if !is_last { "├── " } else { "└── " }, f);
}
