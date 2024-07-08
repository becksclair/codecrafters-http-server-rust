use std::io::prelude::*;
use std::str;
use std::net::{TcpListener, TcpStream};

fn write_response(mut stream: TcpStream, msg: &str) {
    if let Err(err) = stream.write(msg.as_bytes()) {
        eprintln!("Fatal error writing from stream: {:?}", err);
    }
}

fn handle_connection(mut stream: TcpStream) {
    // Create a buffer on the stack with a fixed size
    let mut buffer = [0; 512];

    match stream.read(&mut buffer) {
        Ok(_) => {
            if let Ok(input_request) = str::from_utf8(&buffer) {
                let input_request = input_request.trim_end_matches(char::from(0)).to_string();
                println!("Buffer message: {}", input_request);

                let mut req_parts = input_request.split_whitespace();
                let _verb = req_parts.next(); // Unused for now

                let path = req_parts.next().unwrap_or("");
                println!("Requested path: {}", path);

                let response = match path {
                    "/" => "HTTP/1.1 200 OK\r\n\r\n",
                    _ => "HTTP/1.1 404 Not Found\r\n\r\n",
                };

                write_response(stream, response);
            } else {
                eprintln!("Error converting buffer to string");
            }
        }
        Err(err) => {
            eprintln!("Fatal error reading from stream: {:?}", err);
        }
    }
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
