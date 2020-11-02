#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub use windows::*;
#[cfg(unix)]
mod unix;
#[cfg(unix)]
pub use unix::*;

///Interface to extract acceptable file descriptor for use with `Selector`
pub trait AsRawFd {
    ///Returns raw fd.
    fn as_raw_fd(&self) -> RawFd;
}
