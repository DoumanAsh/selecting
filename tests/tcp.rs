use selecting::Selector;

use std::thread;

use std::io::{self, Read, Write};
use std::net::{TcpStream, TcpListener, Shutdown, SocketAddr, SocketAddrV4, Ipv4Addr};

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
    const LOCALHOST: SocketAddr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 65000));
    let mut selector = Selector::new();

    let (client_notifier, client_notification) = std::sync::mpsc::channel();
    let (server_notifier, server_notification) = std::sync::mpsc::channel();
    let server = thread::spawn(move || {
        let listener = TcpListener::bind(LOCALHOST).expect("Couldn't bind to the address...");
        // Notify the client that the server is ready
        client_notifier.send(()).expect("send client notification");
        let s1 = listener.accept().expect("Couldn't accept the connection...").0;
        let mut s2 = listener.accept().expect("Couldn't accept the connection...").0;
        let mut s3 = listener.accept().expect("Couldn't accept the connection...").0;

        s1.set_nonblocking(true).expect("set_nonblocking call failed");
        s2.set_nonblocking(true).expect("set_nonblocking call failed");
        s3.set_nonblocking(true).expect("set_nonblocking call failed");

        s2.write(b"Hello from s2").expect("Couldn't write to the socket...");
        s3.write(b"Hello from s3").expect("Couldn't write to the socket...");

        // Notify the client that the server has sent the data
        client_notifier.send(()).expect("send client notification");
        // Wait for the client to receive the data
        server_notification.recv().expect("receive server notification");

        s1.shutdown(Shutdown::Both).expect("Couldn't shutdown the socket...");
        s2.shutdown(Shutdown::Both).expect("Couldn't shutdown the socket...");
        s3.shutdown(Shutdown::Both).expect("Couldn't shutdown the socket...");
    });

    // Wait for the server to start
    client_notification.recv().expect("receive server notifcation");
    let s1 = TcpStream::connect(LOCALHOST).expect("Couldn't connect to the server...");
    let s2 = TcpStream::connect(LOCALHOST).expect("Couldn't connect to the server...");
    let s3 = TcpStream::connect(LOCALHOST).expect("Couldn't connect to the server...");

    s1.set_nonblocking(true).expect("set_nonblocking call failed");
    s2.set_nonblocking(true).expect("set_nonblocking call failed");
    s3.set_nonblocking(true).expect("set_nonblocking call failed");

    // Wait for the server to send the data
    client_notification.recv().expect("receive server notifcation");

    loop {
        selector.add_read(&s1);
        selector.add_read(&s2);
        selector.add_read(&s3);
        let result = selector.select().expect("To try select");
        selector.clear_read();
        let len = result.len();
        assert!(len <= 2);
        if len == 2 {
            break;
        }
    }

    // Notify the server that the client has received the data
    server_notifier.send(()).expect("send server notifcation");

    server.join().expect("Couldn't join the server thread...");

    selector.add_read(&s1);
    selector.add_read(&s2);
    selector.add_read(&s3);
    let result = selector.select().expect("To try select");
    //Shutdown should trigger read
    assert_eq!(result.len(), 3);
    assert!(result.is_read(&s1));
    assert!(result.is_read(&s2));
    assert!(result.is_read(&s3));
}
