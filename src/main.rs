use std::{
    fs::{read_dir, FileType, ReadDir},
    io::Result,
    path::{Path, PathBuf},
};

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
            inner: Options { depth: None },
        }
    }

    pub fn build(self) -> Options {
        self.inner
    }

    pub fn with_depth(mut self, depth: usize) -> Self {
        self.inner.depth = Some(depth);
        self
    }
}

struct File {
    name: String,
    file_type: MyFileType,
    path: PathBuf,
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
    let options = OptionsBuilder::new().with_depth(3).build();
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
            if let MyFileType::Symlink = f.file_type {
                format!("{} -> {}", f.name, "Symlink lmao")
            } else {
                f.name
            }
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
