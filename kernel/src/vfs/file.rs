use super::*;

lazy_static! {
    pub static ref LOADER: Mutex<FileLoader> = Mutex::new(FileLoader::new());
}


pub struct FileHandle {
    pos: u64,
    content: Vec<u8>,
}

impl FileHandle {
    pub fn read(&mut self, buffer: *mut u8, size: u64) {
    }
}

pub struct FileLoader {
    handles: BTreeMap<usize, FileHandle>,
}

impl FileLoader {
    pub fn new() -> FileLoader {
        FileLoader {
            handles: BTreeMap::new(),
        }
    }

    pub fn open(&mut self, path: &str) {
        let mut root = ROOT.lock();
    }
}


