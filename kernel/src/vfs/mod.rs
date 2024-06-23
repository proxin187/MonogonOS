pub mod file;
pub mod ata;

use lazy_static::lazy_static;
use spin::mutex::Mutex;

use alloc::string::{String, ToString};
use alloc::collections::BTreeMap;
use alloc::vec::Vec;

lazy_static! {
    static ref ROOT: Mutex<Entry> = Mutex::new(Entry::new_dir());
}


#[derive(Debug)]
pub enum VfsError {
    DirEntry,
    NotFound,
}

#[derive(Clone)]
pub enum Entry {
    Directory {
        entries: BTreeMap<String, Entry>,
    },
    File {
        content: Vec<u8>,
    },
}

impl Entry {
    pub fn new_dir() -> Entry {
        Entry::Directory {
            entries: BTreeMap::new(),
        }
    }

    pub fn new_file() -> Entry {
        Entry::File {
            content: Vec::new(),
        }
    }

    pub fn inspect<F>(&mut self, mut path: impl Iterator<Item = String>, f: F) -> Result<(), VfsError> where F: Fn(&mut Entry) -> Result<(), VfsError> {
        match self {
            Entry::Directory { entries } => {
                if let Some(name) = path.next() {
                    let entry = entries.get_mut(&name).ok_or::<VfsError>(VfsError::NotFound)?;

                    entry.inspect(path, f)?;
                } else {
                    f(self)?;
                }
            },
            Entry::File { .. } => {
                f(self)?;
            },
        }

        Ok(())
    }

    pub fn retrieve(&mut self, path: &str) {
    }

    pub fn make<F>(&mut self, path: &str, new: F) -> Result<(), VfsError> where F: Fn() -> Entry {
        let mut path = path.split('/')
            .map(|x| x.to_string())
            .filter(|x| !x.is_empty())
            .collect::<Vec<String>>();

        let name = path.pop().unwrap_or_default();

        self.inspect(path.iter().cloned(), |entry| {
            match entry {
                Entry::Directory { entries } => {
                    entries.insert(name.clone(), new());

                    Ok(())
                },
                Entry::File { .. } => Err(VfsError::DirEntry),
            }
        })?;

        Ok(())
    }
}

pub fn init() -> Result<(), VfsError> {
    let mut root = ROOT.lock();

    root.make("/tty", || Entry::new_dir())?;

    root.make("/tty/stdout", || Entry::new_file())?;
    root.make("/tty/stdin", || Entry::new_file())?;

    Ok(())
}


