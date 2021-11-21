#[warn(overflowing_literals)]
use std::collections::HashMap;
use std::ffi::c_void;
use std::{io, mem};
use std::io::BufReader;
use std::io::prelude::*;
use std::net::{Shutdown, TcpListener, TcpStream};
use std::os::unix::io::{AsRawFd, RawFd};
use std::time::Duration;
use crate::epoll::ffi::{close, epoll_create1, epoll_ctl, EPOLL_CTL_ADD, EPOLL_CTL_DEL, EPOLL_CTL_MOD, epoll_wait, EPOLLET, EPOLLHUP, EPOLLIN, EPOLLONESHOT, EPOLLOUT, EPOLLRDHUP, Event, F_GETFD, F_SETFD, F_SETFL, fcntl, O_NONBLOCK, READ_FLAGS, WRITE_FLAGS, read};

mod epoll;

static RESPONSE: &str = "HTTP/1.1 200
Content-Type: text/html; charset=UTF-8
Content-Length: 5
Cache-Control: no-cache, no-transform

epoll";

fn set_nonblocking(fd: i32) -> () {
    let res = unsafe { fcntl(fd, F_SETFL, O_NONBLOCK) };
    if res < 0 {
        panic!("FCNTL ERROR");
    }
    ()
}

fn main() -> io::Result<()> {
    let epoll_fd = unsafe { epoll_create1(0) };

    let mut token: i32 = 0;

    if epoll_fd < 0 {
        panic!("create epoll");
    }

    let server = TcpListener::bind("127.0.0.1:8080")?;
    let server_fd = server.as_raw_fd();
    set_nonblocking(server_fd);

    let mut events: Vec<Event> = Vec::with_capacity(1024);

    let mut sockets: HashMap<i32, TcpStream> = HashMap::new();
    let mut requests: HashMap<i32, Vec<u8>> = HashMap::new();

    unsafe { epoll_ctl(epoll_fd, EPOLL_CTL_ADD, server_fd, &mut Event { events: (EPOLLIN | EPOLLOUT | EPOLLET) as u32, u64: token as u64 }) };

    let mut buffer = [0 as u8; 1024];

    loop {
        events.clear();
        let wait = unsafe { epoll_wait(epoll_fd, events.as_mut_ptr(), 1024, -1) };
        unsafe { events.set_len(wait as usize) };

        for event in &events {
            println!("EVENT RECEIVED: {:?}", event);
            match event.u64 {
                0 => {
                    match server.accept() {
                        Ok((stream, addr)) => {
                            let stream_fd = stream.as_raw_fd();
                            set_nonblocking(stream_fd);
                            token += 1;
                            println!("IN HERE");
                            unsafe { epoll_ctl(epoll_fd, EPOLL_CTL_ADD, stream_fd, &mut Event { events: (EPOLLIN | EPOLLET | EPOLLRDHUP | EPOLLHUP) as u32, u64: token as u64 }) };
                            sockets.insert(token, stream);
                            requests.insert(token, Vec::with_capacity(192));
                        }
                        Err(e) => {
                            println!("ACCEPT ERROR: {:?}", e);
                        }
                    }
                    unsafe { epoll_ctl(epoll_fd, EPOLL_CTL_MOD, server_fd, &mut Event { events: (EPOLLIN | EPOLLOUT | EPOLLET) as u32, u64: 0 }) };
                }
                _ => {
                    match event.events {
                        v => {
                            if v as i32 & EPOLLIN == EPOLLIN {
                                loop {
                                    let mut reader = sockets.get_mut(&(event.u64 as i32)).unwrap().read(&mut buffer);
                                    match reader {
                                        Ok(0) => {
                                            println!("ZERO BUFFER");
                                            let stream = sockets.get_mut(&(event.u64 as i32)).unwrap();
                                            let stream_fd = stream.as_raw_fd();
                                            stream.shutdown(Shutdown::Both);
                                            unsafe { epoll_ctl(epoll_fd, EPOLL_CTL_DEL, stream_fd, std::ptr::null_mut()) };
                                            sockets.remove(&(event.u64 as i32)).unwrap();
                                            break;
                                        }
                                        
                                        Ok(n) => {
                                            let req = requests.get_mut(&((event.u64 as i32) as i32)).unwrap();
                                            for b in &buffer[0..n] {
                                                req.push(*b);
                                            }
                                            println!("BYTE SIZE: {:?}", n);
                                            break;
                                        }

                                        Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                                            break;
                                        }

                                        Err(e) => {
                                            println!("READ ERROR: {:?}", e);
                                            break;
                                        }
                                    }
                                }

                                let socket = sockets.get_mut(&(event.u64 as i32));
                                match socket {
                                    Some(s) => {
                                        unsafe { epoll_ctl(epoll_fd, EPOLL_CTL_MOD, s.as_raw_fd(), &mut Event { events: (EPOLLOUT | EPOLLET) as u32, u64: event.u64 }) };
                                        println!("FINISHED");
                                    }
                                    None => {}
                                }
                            }
                            if v as i32 & EPOLLOUT == EPOLLOUT {
                                println!("WRITE");
                                let socket = sockets.get_mut(&(event.u64 as i32)).unwrap();
                                requests.get_mut(&(event.u64 as i32)).unwrap().clear();
                                socket.write(RESPONSE.as_bytes()).unwrap();
                                unsafe { epoll_ctl(epoll_fd, EPOLL_CTL_MOD, socket.as_raw_fd(), &mut Event { events: (EPOLLIN | EPOLLET) as u32, u64: event.u64 }) };
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}