use crate::epoll::epoll::ffi::{epoll_ctl, EPOLL_CTL_ADD, Event, READ_FLAGS};

pub mod ffi {
    use std::ffi::c_void;
    use std::os::raw::{c_int};

    pub const EPOLL_CTL_ADD: i32 = 0x1;
    pub const EPOLL_CTL_DEL: i32 = 0x2;
    pub const EPOLL_CTL_MOD: i32 = 0x3;
    pub const EPOLLIN: i32 = 0x1;
    pub const EPOLLOUT: i32 = 0x4;
    pub const EPOLLRDHUP: i32 = 0x2000;
    pub const EPOLLHUP: i32 = 0x10;
    pub const O_NONBLOCK: i32 = 2048;
    pub const F_SETFL: i32 = 4;
    #[allow(overflowing_literals)]
    pub const EPOLLET: i32 = 0x80000000;
    pub const EPOLLONESHOT: i32 = 0x40000000;
    pub const READ_FLAGS: i32 = EPOLLONESHOT | EPOLLIN;
    pub const WRITE_FLAGS: i32 = EPOLLONESHOT | EPOLLOUT;
    pub const F_GETFD: i32 = 1;
    pub const F_SETFD: i32 = 2;

    #[derive(Debug)]
    #[repr(C, packed)]
    pub struct Event {
        pub events: u32,
        pub u64: u64,
    }

    #[link(name = "c")]
    extern "C" {
        pub fn epoll_create1(size: i32) -> i32;
        pub fn epoll_ctl(epoll_fd: i32, op: i32, fd: i32, event: *mut Event) -> i32;
        pub fn close(fd: i32) -> i32;
        pub fn epoll_wait(epoll_fd: i32, events: *mut Event, max_events: i32, timeout: i32) -> i32;
        pub fn fcntl(fd: i32, cmd: i32, ...) -> i32;
    }

    extern "system" {
        pub fn read(fd: c_int, buffer: *mut c_void, count: isize) -> isize;
    }
}