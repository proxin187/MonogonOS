use crate::vfs::file;

use core::ffi::CStr;


pub enum SyscallError {
    InvalidPath,
    Unknown,
}

#[non_exhaustive]
pub struct Kind;

impl Kind {
    const READ:  i64 = 0;
    const WRITE: i64 = 1;
    const OPEN:  i64 = 2;
}

pub struct Syscall {
    pub args: [i64; 4],
}

impl Syscall {
    pub fn new() -> Syscall {
        Syscall {
            args: [0; 4],
        }
    }

    pub fn perform(&self) -> Result<(), SyscallError> {
        match self.args[0] {
            Kind::READ => {
                Ok(())
            },
            Kind::WRITE => {
                Ok(())
            },
            Kind::OPEN => {
                let mut loader = file::LOADER.lock();

                unsafe {
                    let path = CStr::from_ptr(self.args[1] as *const i8);

                    loader.open(path.to_str().map_err(|_| SyscallError::InvalidPath)?);
                }

                Ok(())
            },
            _ => Err(SyscallError::Unknown),
        }
    }
}


