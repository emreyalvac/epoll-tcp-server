use std::net::TcpStream;

pub trait TRequestContext {
    fn new(token: i32, stream: TcpStream, buffer: Vec<u8>) -> Self;
    fn mut_buffer(&mut self) -> &mut Vec<u8>;
}

pub struct RequestContext {
    token: i32,
    stream: TcpStream,
    buffer: Vec<u8>,
}

impl TRequestContext for RequestContext {
    fn new(token: i32, stream: TcpStream, buffer: Vec<u8>) -> Self {
        Self {
            token,
            stream,
            buffer,
        }
    }

    fn mut_buffer(&mut self) -> &mut Vec<u8> {
        &mut self.buffer
    }
}