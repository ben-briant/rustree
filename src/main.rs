use std::{
    fs::{read_dir, FileType},
    io::Result,
    path::{Path, PathBuf}, fmt::Display, os::unix::prelude::PermissionsExt,
};
use colored::Colorize;
use clap::Parser;

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
}

#[derive(Clone, Copy)]
struct Options {
    pub depth: Option<usize>,
}

struct OptionsBuilder {
    inner: Options,
}

impl OptionsBuilder {
    pub fn new() -> Self {
        Self {
            inner: Options { depth: None},
        }
    }

    pub fn build(self) -> Options {
        self.inner
    }

    pub fn with_depth(&mut self, depth: usize)  {
        self.inner.depth = Some(depth);
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

        let colored =
        match (&self.file_type, self.is_executable) {
            (MyFileType::File, false) => self.name.white(),
            (MyFileType::File, true) => self.name.green(),
            (MyFileType::Dir, _) => self.name.blue(),
            (MyFileType::Symlink, _) => self.name.cyan(),
            (MyFileType::BlockDevice, _) => self.name.yellow(),
            (MyFileType::CharDevice, _) => self.name.red(),
            (MyFileType::Fifo, _) => self.name.magenta(),
            (MyFileType::Socket, _) => self.name.black(),
        };
        write!(f, "{colored}")
    }
}

enum MyFileType {
    File,
    Dir,
    Symlink,
    BlockDevice,
    CharDevice,
    Fifo,
    Socket,
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

fn main() -> Result<()> {
    let args = Args::parse();
    if args.no_colors {
        colored::control::set_override(false);
    }
    let mut options_builder = OptionsBuilder::new();
    if let Some(depth) = args.depth {
        options_builder.with_depth(depth);
    }
    let options = options_builder.build();
    let depth = 0;
    traverse_layer(Path::new("."), depth, "".to_string(), options)?;
    Ok(())
}

fn traverse_layer(path: &Path, depth: usize, prefix: String, options: Options) -> Result<()> {
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
        .peekable();
    while let Some(f) = file_it.next() {
        println!(
            "{}{}{}",
            prefix,
            if file_it.peek().is_some() {
                "├── "
            } else {
                "└── "
            },
            f
        );
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
