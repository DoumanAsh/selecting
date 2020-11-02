use selecting::Selector;

use std::io::{self, Read};
use std::net::TcpStream;

#[test]
pub fn should_work_tcp_stream() {
    let mut selector = Selector::new();

    let mut stream = TcpStream::connect("www.google.com:80").expect("Couldn't connect to the server...");
    stream.set_nonblocking(true).expect("set_nonblocking call failed");

    let mut buffer: [u8; 1024] = [0; 1024];
    let error = stream.read(&mut buffer).unwrap_err();
    assert_eq!(error.kind(), io::ErrorKind::WouldBlock);

    selector.add_read(&stream);

    let result = selector.try_select().expect("To try select");
    assert_eq!(result.len(), 0);
    assert!(!result.is_read(&stream));

    selector.clear_read();
    selector.add_write(&stream);
    let result = selector.try_select().expect("To try select");
    assert_eq!(result.len(), 1);
    assert!(!result.is_read(&stream));
    assert!(result.is_write(&stream));
}
