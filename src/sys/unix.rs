use super::AsRawFd;

use std::io;
use core::mem::MaybeUninit;
use core::{time, cmp, ptr};

pub type RawFd = std::os::unix::io::RawFd;
///Limit on file descriptors in select's set
pub const FD_LIMIT: usize = libc::FD_SETSIZE as usize;

impl<T: std::os::unix::io::AsRawFd> AsRawFd for T {
    #[inline(always)]
    fn as_raw_fd(&self) -> RawFd {
        std::os::unix::io::AsRawFd::as_raw_fd(self)
    }
}

#[derive(Clone, Copy)]
pub struct FdSet {
    max_fd: RawFd,
    count: usize,
    inner: MaybeUninit<libc::fd_set>,
}

impl FdSet {
    #[inline]
    pub fn new() -> Self {
        let mut inner = MaybeUninit::uninit();
        unsafe {
            libc::FD_ZERO(inner.as_mut_ptr());
        }

        Self {
            max_fd: 0,
            count: 0,
            inner,
        }
    }

    #[inline]
    pub fn add<T: AsRawFd>(&mut self, source: &T) {
        self.count += 1;
        self.max_fd = cmp::max(source.as_raw_fd(), self.max_fd);
        unsafe {
            libc::FD_SET(source.as_raw_fd(), self.inner.as_mut_ptr())
        }
    }

    #[inline]
    pub fn is_present<T: AsRawFd>(&self, source: &T) -> bool {
        unsafe {
            libc::FD_ISSET(source.as_raw_fd(), self.inner.as_ptr() as _)
        }
    }

    #[inline]
    pub fn clear(&mut self) {
        self.count = 0;
        self.max_fd = 0;
        unsafe {
            libc::FD_ZERO(self.inner.as_mut_ptr());
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.count
    }

    #[inline]
    pub fn raw(&self) -> Self {
        self.clone()
    }
}

pub fn select(read: &mut FdSet, write: &mut FdSet) -> io::Result<usize> {
    let nfds = cmp::max(read.max_fd, write.max_fd);

    let read_fd = match read.max_fd {
        0 => ptr::null_mut(),
        _ => read.inner.as_mut_ptr(),
    };

    let write_fd = match write.max_fd {
        0 => ptr::null_mut(),
        _ => write.inner.as_mut_ptr(),
    };

    let result = unsafe {
        libc::select(nfds, read_fd, write_fd, ptr::null_mut(), ptr::null_mut())
    };

    match result {
        -1 => Err(io::Error::last_os_error()),
        _ => Ok(result as usize)
    }
}

pub fn select_timeout(read: &mut FdSet, write: &mut FdSet, timeout: time::Duration) -> io::Result<usize> {
    use core::convert::TryInto;

    let mut timeout = libc::timeval {
        tv_sec: match timeout.as_secs().try_into() {
            Ok(secs) => secs,
            Err(_) => return Err(io::Error::new(io::ErrorKind::InvalidInput, "Duration as seconds doesn't fit timeval")),
        },
        tv_usec: timeout.subsec_micros() as _,
    };

    let nfds = cmp::max(read.max_fd, write.max_fd) + 1;

    let read_fd = match read.max_fd {
        0 => ptr::null_mut(),
        _ => read.inner.as_mut_ptr(),
    };

    let write_fd = match write.max_fd {
        0 => ptr::null_mut(),
        _ => write.inner.as_mut_ptr(),
    };

    let result = unsafe {
        libc::select(nfds, read_fd, write_fd, ptr::null_mut(), &mut timeout)
    };

    match result {
        -1 => Err(io::Error::last_os_error()),
        _ => Ok(result as usize)
    }
}
