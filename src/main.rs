use std::io::prelude::*;
use std::path::Path;
use std::{env, fs, str};
use std::net::{TcpListener, TcpStream};

fn write_response(mut stream: TcpStream, msg: &str) {
    if let Err(err) = stream.write_all(msg.as_bytes()) {
        eprintln!("Error writing to stream: {:?}", err);
    }
}

fn handle_connection(mut stream: TcpStream, file_dir: String) {
    // Create a buffer on the stack with a fixed size
    let mut buffer = [0; 512];

    match stream.read(&mut buffer) {
        Ok(_) => {
            if let Ok(input_request) = str::from_utf8(&buffer) {
                let input_request = input_request.trim_end_matches(char::from(0)).to_string();

                let mut req_parts = input_request.split("\r\n");
                let mut request = req_parts.next().unwrap().split_whitespace();

                let verb = request.next().unwrap_or("");
                let path = request.next().unwrap_or("");

                println!("Verb: {}", verb);
                println!("Requested path: {}", path);

                let mut header_ua             = String::new();
                let mut header_content_length = String::new();
                let mut header_content_type   = String::new();

                let mut request_body = String::new();

                req_parts.for_each(|part| {
                    if part.starts_with("User-Agent") {
                        header_ua = part.split_whitespace().last().unwrap_or("").to_string();
                        println!("Header: {}", header_ua);
                        return;
                    }
                    if part.starts_with("Content-Length") {
                        header_content_length = part.split_whitespace().last().unwrap_or("").to_string();
                        println!("Content-Length: {}", header_content_length);
                    }
                    if part.starts_with("Content-Type") {
                        header_content_type = part.split_whitespace().last().unwrap_or("").to_string();
                        println!("Content-Type: {}", header_content_type);
                    }

                    if !part.trim().is_empty() {
                        request_body = part.to_string();
                    }
                    // println!("Part: {}", part);
                });

                println!("Request Body: {}", request_body);

                let response = if path.starts_with("/echo/") {
                    let echo_str = &path[6..]; // Extract the string after "/echo/"
                    format!("HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}", echo_str.len(), echo_str)
                }
                else if path.starts_with("/user-agent") {
                    format!("HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}", header_ua.len(), header_ua)
                }
                else if path.starts_with("/files/") {
                    let file_str = &path[7..]; // Extract the string after "/files/"
                    let file_path = [file_dir, file_str.to_string()].concat();

                    // Write file contents
                    match verb {
                        "POST" => {
                            fs::write(file_path, request_body).expect("Should have been able to write the file");
                            "HTTP/1.1 201 Created\r\n\r\n".to_string()
                        }
                        "GET" => {

                            // Check the file exists
                            match Path::new(&file_path).exists() {
                                true => {
                                    let contents = fs::read_to_string(file_path).expect("Should have been able to read the file");
                                    format!("HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: {}\r\n\r\n{}", contents.len(), contents)
                                }
                                false => {
                                    println!("Error file didn't exist.");
                                    "HTTP/1.1 404 Not Found\r\n\r\n".to_string()
                                }
                            }
                        }
                        _ => "HTTP/1.1 404 Not Found\r\n\r\n".to_string()
                    }

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
                let file_dir = env::args().nth(2).unwrap_or_else(|| "".to_string());
                handle_connection(stream, file_dir);
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
