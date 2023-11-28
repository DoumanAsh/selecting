use selecting::Selector;

use std::thread::{self, sleep};

use std::io::{self, Read, Write};
use std::net::{TcpStream, TcpListener, Shutdown};
use std::os::fd::AsRawFd;
use std::time::Duration;

#[test]
pub fn should_work_tcp_stream() {
    let mut selector = Selector::new();

    let mut stream = TcpStream::connect("www.google.com:80").expect("Couldn't connect to the server...");
    stream.set_nonblocking(true).expect("set_nonblocking call failed");

    let mut buffer: [u8; 1024] = [0; 1024];
    let error = stream.read(&mut buffer).unwrap_err();
    assert_eq!(error.kind(), io::ErrorKind::WouldBlock);

    selector.add_read(&stream);
    selector.add_except(&stream);

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

#[test]
pub fn should_work_with_multiple_fds() {
    let mut selector = Selector::new();

    let server = thread::spawn(|| {
        let listener = TcpListener::bind("localhost:1234").expect("Couldn't bind to the address...");
        let s1 = listener.accept().expect("Couldn't accept the connection...").0;
        let mut s2 = listener.accept().expect("Couldn't accept the connection...").0;
        let mut s3 = listener.accept().expect("Couldn't accept the connection...").0;

        s1.set_nonblocking(true).expect("set_nonblocking call failed");
        s2.set_nonblocking(true).expect("set_nonblocking call failed");
        s3.set_nonblocking(true).expect("set_nonblocking call failed");

        s2.write(b"Hello from s2").expect("Couldn't write to the socket...");
        s3.write(b"Hello from s3").expect("Couldn't write to the socket...");

        sleep(Duration::from_millis(100));

        s1.shutdown(Shutdown::Both).expect("Couldn't shutdown the socket...");
        s2.shutdown(Shutdown::Both).expect("Couldn't shutdown the socket...");
        s3.shutdown(Shutdown::Both).expect("Couldn't shutdown the socket...");
    });

    
    let s1 = TcpStream::connect("localhost:1234").expect("Couldn't connect to the server...");
    let s2 = TcpStream::connect("localhost:1234").expect("Couldn't connect to the server...");
    let s3 = TcpStream::connect("localhost:1234").expect("Couldn't connect to the server...");
    
    s1.set_nonblocking(true).expect("set_nonblocking call failed");
    s2.set_nonblocking(true).expect("set_nonblocking call failed");
    s3.set_nonblocking(true).expect("set_nonblocking call failed");
    
    selector.add_read(&s1);
    selector.add_read(&s2);
    selector.add_read(&s3);
    
    println!("s1 fd: {}", s1.as_raw_fd());
    println!("s2 fd: {}", s2.as_raw_fd());
    println!("s3 fd: {}", s3.as_raw_fd());
    
    let result = selector.select().expect("To try select");
    assert_eq!(result.len(), 2);
    assert!(!result.is_read(&s1));
    assert!(result.is_read(&s2));
    assert!(result.is_read(&s3));

    server.join().expect("Couldn't join the server thread...");
}
