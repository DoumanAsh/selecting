use super::AsRawFd;

use core::{ptr, time};
use std::io;
use std::os::windows::io::RawSocket;

pub type RawFd = RawSocket;
///Limit on file descriptors in select's set
pub const FD_LIMIT: usize = 64;

impl<T: std::os::windows::io::AsRawSocket> AsRawFd for T {
    #[inline(always)]
    fn as_raw_fd(&self) -> RawFd {
        self.as_raw_socket()
    }
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct FdSet {
    count: winapi::um::winsock2::u_int,
    fds: [RawSocket; 64],
}

impl FdSet {
    #[inline]
    pub fn new() -> Self {
        Self {
            count: 0,
            fds: unsafe {
                core::mem::MaybeUninit::zeroed().assume_init()
            }
        }
    }

    #[inline]
    pub fn add<T: AsRawFd>(&mut self, source: &T) {
        let idx = self.count as usize;
        self.fds[idx] = source.as_raw_fd();
        self.count += 1;
    }

    #[inline]
    pub fn clear(&mut self) {
        for fd in self.fds[..self.count as usize].iter_mut() {
            *fd = 0;
        }
        self.count = 0;
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.count as usize
    }

    #[inline]
    pub fn is_present<T: AsRawFd>(&self, source: &T) -> bool {
        let expected = source.as_raw_fd();
        for fd in self.fds[..self.count as usize].iter() {
            if *fd == expected {
                return true;
            }
        }

        false
    }
}

pub fn select(read: &mut FdSet, write: &mut FdSet, except: &mut FdSet) -> io::Result<usize> {
    use winapi::um::winsock2::SOCKET_ERROR;

    let read_fd = match read.len() {
        0 => ptr::null_mut(),
        _ => read as *mut _ as *mut _,
    };

    let write_fd = match write.len() {
        0 => ptr::null_mut(),
        _ => write as *mut _ as *mut _,
    };

    let except_fd = match except.len() {
        0 => ptr::null_mut(),
        _ => except as *mut _ as *mut _
    };

    let result = unsafe {
        winapi::um::winsock2::select(0, read_fd, write_fd, except_fd, ptr::null_mut())
    };

    match result {
        SOCKET_ERROR => Err(io::Error::last_os_error()),
        result => Ok(result as usize)
    }
}

pub fn select_timeout(read: &mut FdSet, write: &mut FdSet, except: &mut FdSet, timeout: time::Duration) -> io::Result<usize> {
    use core::convert::TryInto;
    use winapi::um::winsock2::SOCKET_ERROR;
    use winapi::um::winsock2::timeval;

    let timeout = timeval {
        tv_sec: match timeout.as_secs().try_into() {
            Ok(secs) => secs,
            Err(_) => return Err(io::Error::new(io::ErrorKind::InvalidInput, "Duration as seconds doesn't fit timeval")),
        },
        tv_usec: timeout.subsec_micros() as _,
    };

    let read_fd = match read.len() {
        0 => ptr::null_mut(),
        _ => read as *mut _ as *mut _,
    };

    let write_fd = match write.len() {
        0 => ptr::null_mut(),
        _ => write as *mut _ as *mut _,
    };

    let except_fd = match except.len() {
        0 => ptr::null_mut(),
        _ => except as *mut _ as *mut _
    };

    let result = unsafe {
        winapi::um::winsock2::select(0, read_fd, write_fd, except_fd, &timeout)
    };

    if result == SOCKET_ERROR {
        Err(io::Error::last_os_error())
    } else {
        Ok(result as usize)
    }
}
