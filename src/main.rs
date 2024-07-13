// use std::io::prelude::*;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use std::path::Path;
use std::{env, fs, str};

enum ResponseType {
    Ok,
    Created,
    NotFound,
    Error
}

#[derive(Debug, PartialEq)]
struct Request {
    method: String,
    path: String,
    headers: Vec<(String, String)>,
    body: Option<String>,
}

impl Request {
    fn new(raw_request: &str) -> Request {
        let clean_request = raw_request.trim_end_matches(char::from(0));

        let mut req_sections = clean_request.lines();
        let req_line = req_sections.next().unwrap();

        let mut req_parts = req_line.split_whitespace();

        let method  = req_parts.next().unwrap();
        let path    = req_parts.next().unwrap();
        // let version = req_parts.next().unwrap();

        let mut body = None;
        let mut headers = Vec::new();

        while let Some(line) = req_sections.next() {
            if line.is_empty() {
                // End of headers, the rest is the body
                body = Some(req_sections.collect::<Vec<&str>>().join("\n"));
                break;
            }

            let (key, value) = line.split_once(": ").unwrap();
            headers.push((key.to_lowercase().to_string(), value.to_string()));
        }

        Request {
            method: method.to_string(),
            path: path.to_string(),
            headers,
            body,
        }
    }
}

async fn write_response(mut stream: TcpStream, msg: &str) {
    if let Err(err) = stream.write_all(msg.as_bytes()).await {
        eprintln!("Error writing to stream: {:?}", err);
    }
}

fn build_response(request: &Request, response_type: ResponseType, response_body: Option<String>, response_headers: Option<Vec<(String, String)>>) -> String {
    let mut response = String::new();

    match response_type {
        ResponseType::Ok => {
            response.push_str("HTTP/1.1 200 OK\r\n");
        },
        ResponseType::Created => {
            response.push_str("HTTP/1.1 201 Created\r\n");
        },
        ResponseType::NotFound => {
            response.push_str("HTTP/1.1 404 Not Found\r\n");
        },
        ResponseType::Error => {
            response.push_str("HTTP/1.1 500 Internal Server Error\r\n");
        },
    }

    if let Some(encoding) = request.headers.iter().find(|(key, val)| {
        key == "accept-encoding" && val.split(',').any(|v| v.trim().to_lowercase() == "gzip")
    }) {
        let accepted_encodings = encoding.1.split(',').map(|v| v.trim().to_lowercase()).collect::<Vec<String>>();
        let valid_encodings = ["gzip", "deflate", "br"];
        
        for enc in accepted_encodings.iter() {
            if valid_encodings.contains(&enc.as_str()) {
                response.push_str(format!("Content-Encoding: {}\r\n", enc).as_str());
                println!("Found valid encoding: {}", enc);
            }
        }
    }

    if let Some(headers) = response_headers {
        for (key, value) in headers {
            response.push_str(&key);
            response.push_str(": ");
            response.push_str(&value);
            response.push_str("\r\n");
        }
    }

    if let Some(body) = response_body {
        println!("Body: {}", body);
        println!("Body Length: {}", body.len());

        response.push_str("Content-Type: text/plain\r\n");
        response.push_str("Content-Length: ");
        response.push_str(&body.len().to_string());
        response.push_str("\r\n\r\n");
        response.push_str(&body);
    }
    else {
        response.push_str("\r\n");
    }
    response
}

async fn handle_connection(mut stream: TcpStream, file_dir: String) {
    // Create a buffer on the stack with a fixed size
    let mut buffer = [0; 512];

    match stream.read(&mut buffer).await {
        Ok(bytes_read) => {
            if bytes_read == 0 {
                // Connection was closed
                println!("Connection was closed");
                return;
            }
            println!("Bytes read: {}", bytes_read);

            if let Ok(input_request) = str::from_utf8(&buffer) {
                let request = Request::new(input_request);

                let response =
                    if request.path.starts_with("/echo/") {
                        let echo_str = &request.path[6..]; // Extract the string after "/echo/"
                        build_response(&request, ResponseType::Ok, Some(echo_str.to_string()), None)
                    }
                    else if request.path.starts_with("/user-agent") {
                        let ua_header = request.headers.iter().find(|(key, _)| key == "user-agent");

                        match ua_header {
                            Some((_, ua)) => {
                                build_response(&request, ResponseType::Ok, Some(ua.to_string()), None)
                            },
                            None => {
                                build_response(&request, ResponseType::Error, None, None)
                            },
                        }
                    }
                    else if request.path.starts_with("/files/") {
                        let file_str = &request.path[7..]; // Extract the string after "/files/"
                        let file_path = [file_dir, file_str.to_string()].concat();
                    
                        // Write file contents
                        match request.method.as_str() {
                            "POST" => {
                                fs::write(file_path, request.body.as_ref().unwrap()).expect("Should have been able to write the file");
                                build_response(&request, ResponseType::Created, None, None)
                            }
                            "GET" => {
                                // Check the file exists
                                match Path::new(&file_path).exists() {
                                    true => {
                                        let contents = fs::read_to_string(file_path).expect("Should have been able to read the file");
                                        let response_headers = vec![("Content-Type".to_string(), "application/octet-stream".to_string())];
                                        build_response(&request, ResponseType::Ok, Some(contents), Some(response_headers))
                                    }
                                    false => {
                                        println!("Error file didn't exist.");
                                        build_response(&request, ResponseType::NotFound, None, None)
                                    }
                                }
                            }
                            _ => build_response(&request, ResponseType::NotFound, None, None),
                        }
                    }
                else {
                    match request.path.as_str() {
                        "/" => build_response(&request, ResponseType::Ok, Some("Welcome to the home page!".to_string()), None),
                        _ => build_response(&request, ResponseType::NotFound, None, None),
                    }
                    .to_string()
                };

                write_response(stream, &response).await;
            } else {
                eprintln!("Error converting buffer to string");
            }
        }
        Err(err) => {
            eprintln!("Fatal error reading from stream: {:?}", err);
        }
    }
}

#[tokio::main]
async fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:4221").await.unwrap();
    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                println!("accepted new connection");
                let file_dir = env::args().nth(2).unwrap_or_default();
                tokio::spawn(async move {
                    handle_connection(stream, file_dir).await;
                });
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_request() {
        let raw_request =
            "GET /echo/hello HTTP/1.1\r\nUser-Agent: curl/7.83.1\r\nAccept: */*\r\n\r\n";
        let request = Request::new(raw_request);

        assert_eq!(request.method, "GET");
        assert_eq!(request.path, "/echo/hello");
    }

    #[test]
    fn test_response_ok() {
        let request = Request::new("GET /echo/hello HTTP/1.1\r\nUser-Agent: curl/7.83.1\r\nAccept: */*\r\n\r\n");
        let content = "Welcome to the home page!";
        let response = build_response(&request, ResponseType::Ok, Some(content.to_string()), None);      

        assert_eq!(response, format!("HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}\r\n", content.len().to_string(), content));
    }

    #[test]
    fn test_response_not_found() {
        let request = Request::new("GET /echo/hello HTTP/1.1\r\nUser-Agent: curl/7.83.1\r\nAccept: */*\r\n\r\n");
        let response = build_response(&request, ResponseType::NotFound, None, None);      
        assert_eq!(response, "HTTP/1.1 404 Not Found\r\n\r\n");
    }
}
