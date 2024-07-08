use std::io::prelude::*;
use std::str;
use std::net::{TcpListener, TcpStream};

fn write_response(mut stream: TcpStream, msg: &str) {
    if let Err(err) = stream.write_all(msg.as_bytes()) {
        eprintln!("Error writing to stream: {:?}", err);
    }
}

fn handle_connection(mut stream: TcpStream) {
    // Create a buffer on the stack with a fixed size
    let mut buffer = [0; 512];

    match stream.read(&mut buffer) {
        Ok(_) => {
            if let Ok(input_request) = str::from_utf8(&buffer) {
                let input_request = input_request.trim_end_matches(char::from(0)).to_string();

                let mut req_parts = input_request.split("\r\n");
                let mut request = req_parts.next().unwrap().split_whitespace();

                let _verb = request.next(); // Unused for now
                let path = request.next().unwrap_or("");
                println!("Requested path: {}", path);

                let mut header_ua = String::new();
                req_parts.for_each(|header| {
                    if header.starts_with("User-Agent") {
                        header_ua = header.split_whitespace().last().unwrap_or("").to_string();
                        println!("Header: {}", header_ua);
                        return;
                    }
                });

                let response = if path.starts_with("/echo/") {
                    let echo_str = &path[6..]; // Extract the string after "/echo/"
                    format!("HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}", echo_str.len(), echo_str)
                }
                else if path.starts_with("/user-agent") {
                    format!("HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}", header_ua.len(), header_ua)
                }
                else {
                    match path {
                        "/" => "HTTP/1.1 200 OK\r\n\r\nWelcome to the home page!",
                        _ => "HTTP/1.1 404 Not Found\r\n\r\n",
                    }.to_string()
                };

                write_response(stream, &response);
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
