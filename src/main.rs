use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};

fn handle_connection(mut stream: TcpStream) {
    stream.write(b"HTTP/1.1 200 OK\r\n\r\n").unwrap();
}

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("accepted new connection");
                handle_connection(stream);
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
