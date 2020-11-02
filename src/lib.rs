//!Cross-platform wrapper over select.
//!
//!This library provides simple interface over POSIX's `select` enabling you to write
//!very simple async programs using `std` networking primitives.
//!
//!But if you want performance you should look for `tokio` or `mio`

mod sys;

use sys::FdSet;
pub use sys::{AsRawFd};

use std::io;
use core::time;

pub use sys::FD_LIMIT;

///Represents successful select call.
///
///It contains number of fd ready.
///And also set of these ids, allowing to query which fd is ready.
pub struct SelectResult {
    count: usize,
    read: FdSet,
    write: FdSet,
}

impl SelectResult {
    #[inline]
    ///Returns total number of file descriptors ready.
    pub const fn len(&self) -> usize {
        self.count
    }

    #[inline]
    ///Returns whether specified `source` is read ready.
    pub fn is_read<T: AsRawFd>(&self, source: &T) -> bool {
        self.read.is_present(source)
    }

    #[inline]
    ///Returns whether specified `source` is read ready.
    pub fn is_write<T: AsRawFd>(&self, source: &T) -> bool {
        self.write.is_present(source)
    }
}

#[derive(Clone)]
///Select abstraction allowing you to monitor specified sockets.
///
///Selector itself represents set of file descriptors to monitor, as such it is possible
///to copy it.
///Normally select modifies list of file descriptors, but `Selector` always copies existing list,
///avoiding these modifications.
///
///## Performance recommendations
///
///Generally limited to 64 file descriptors, but you should only use select when you have ~10 fds
///otherwise modern API like `kqueue` or `epoll` would yield much better performance.
///
///- Call select only when getting would block from syscall;
///- Limit number of selects, allowing it to accumulate events;
pub struct Selector {
    read: FdSet,
    write: FdSet,
}

impl Selector {
    #[inline]
    ///Creates new instance with no sockets to monitor.
    pub fn new() -> Self {
        Self {
            read: FdSet::new(),
            write: FdSet::new(),
        }
    }

    #[inline]
    ///Adds `source` to monitor for read ops.
    ///
    ///Panics when goes over `FD_LIMIT`
    pub fn add_read<T: AsRawFd>(&mut self, source: &T) {
        assert!(self.read.len() < sys::FD_LIMIT);
        self.read.add(source);
    }

    #[inline]
    ///Adds `source` to monitor for write ops.
    ///
    ///Panics when goes over `FD_LIMIT`
    pub fn add_write<T: AsRawFd>(&mut self, source: &T) {
        assert!(self.write.len() < sys::FD_LIMIT);
        self.write.add(source);
    }

    #[inline]
    ///Removes all fds from read monitoring
    pub fn clear_read(&mut self) {
        self.read.clear();
    }

    #[inline]
    ///Removes all fds from read monitoring
    pub fn clear_write(&mut self) {
        self.write.clear();
    }

    #[inline]
    ///Performs select, awaiting indefinitely until at least one descriptor has changes.
    pub fn select(&self) -> io::Result<SelectResult> {
        let mut result = SelectResult {
            count: 0,
            read: self.read.clone(),
            write: self.write.clone(),
        };

        result.count = sys::select(&mut result.read, &mut result.write)?;
        Ok(result)
    }

    #[inline]
    ///Performs select, checking file descriptors for changes and returning immediately.
    pub fn try_select(&self) -> io::Result<SelectResult> {
        let mut result = SelectResult {
            count: 0,
            read: self.read.clone(),
            write: self.write.clone(),
        };

        result.count = sys::select_timeout(&mut result.read, &mut result.write, time::Duration::from_secs(0))?;
        Ok(result)
    }

    #[inline]
    ///Performs select, awaiting at most `time` until at least one descriptor has changes.
    pub fn select_timeout(&self, time: time::Duration) -> io::Result<SelectResult> {
        let mut result = SelectResult {
            count: 0,
            read: self.read.clone(),
            write: self.write.clone(),
        };

        result.count = sys::select_timeout(&mut result.read, &mut result.write, time)?;
        Ok(result)
    }
}
