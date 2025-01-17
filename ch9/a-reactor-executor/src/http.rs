use std::io::{Write, ErrorKind, Read};

use crate::{Future, future::PollState};

fn get_req(path: &str) -> String {
    format!(
        "GET {path} HTTP/1.1\r\n\
             Host: localhost\r\n\
             Connection: close\r\n\
             \r\n"
    )
}


pub struct Http;

impl Http {
    pub fn get(path: &'static str) -> impl Future<Output = String> {
        HttpGetFuture::new(path)
    }

    fn get_req(path: &str) -> String {
        format!(
            "GET {path} HTTP/1.1\r\n\
             Host: localhost\r\n\
             Connection: close\r\n\
             \r\n"
        )
    }
}

struct HttpGetFuture {
    stream: Option<mio::net::TcpStream>,
    buffer: Vec<u8>,
    path: &'static str,
}

impl HttpGetFuture {
    fn new(path: &'static str) -> Self {
        Self {
            stream: None,
            buffer: vec![],
            path,
        }
    }

    fn write_request(&mut self) {
        let stream = std::net::TcpStream::connect("127.0.0.1:8080").unwrap();
        stream.set_nonblocking(true).unwrap();
        let mut stream = mio::net::TcpStream::from_std(stream);
        stream.write_all(get_req(self.path).as_bytes()).unwrap();
        self.stream = Some(stream);
    }
    
}

impl Future for HttpGetFuture {
    type Output = String;

    fn poll(&mut self) -> PollState<Self::Output> {
        // If this is first time polled, start the operation
        // see: https://users.rust-lang.org/t/is-it-bad-behaviour-for-a-future-or-stream-to-do-something-before-being-polled/61353
        // Avoid dns lookup this time
        if self.stream.is_none() {
            println!("FIRST POLL - START OPERATION");
            self.write_request();
            return PollState::NotReady;
        }

        let mut buff = vec![0u8; 4096];
        loop {
            match self.stream.as_mut().unwrap().read(&mut buff) {
                Ok(0) => {
                    let s = String::from_utf8_lossy(&self.buffer);
                    break PollState::Ready(s.to_string());
                }
                Ok(n) => {
                    self.buffer.extend(&buff[0..n]);
                    continue;
                }
                Err(e) if e.kind() == ErrorKind::WouldBlock => {
                    break PollState::NotReady;
                }

                Err(e) => panic!("{e:?}"),
            }
        }
    }
}