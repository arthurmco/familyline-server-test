use chrono::prelude::*;
use futures::stream::StreamExt;
use regex::Regex;
use std::collections::HashMap;
use tokio::net::{TcpListener, TcpStream};
use tokio::prelude::*;

use std::io::Error;
use serde_json::Value;

mod server;
use server::ServerInfo;

fn format_rfc2616_date(date: DateTime<Utc>) -> String {
    date.format("%a, %d %b %Y %T GMT").to_string()
}

/// Creates the http response, only the header
fn create_http_response(http_code: u32, additional_headers: Vec<String>) -> String {
    let mut lines: Vec<String> = Vec::new();
    let now: DateTime<Utc> = Utc::now();

    let codestr = match http_code {
        101 => "101 Switching Protocols",
        200 => "200 OK",
        400 => "400 Bad Request",
        401 => "401 Unauthorized",
        403 => "403 Forbidden",
        404 => "404 Not Found",
        415 => "415 Unsupported Media Type",
        500 => "500 Internal Server Error",
        505 => "505 HTTP Version Not Supported",
        _ => panic!("unknown http error!"),
    };

    lines.push(format!("HTTP/1.1 {}", codestr));
    lines.push(format!("Date: {}", format_rfc2616_date(now)));
    lines.push("Cache-Control: private, max-age=0".to_string());
    lines.push("Server: familyline-server 0.0.1-test".to_string());

    if http_code >= 200 && http_code <= 299 {
        lines.push("Content-Type: application/json".to_string());
    }

    lines.extend(additional_headers);

    lines.push("".to_string());
    lines.push("".to_string());

    return lines.join("\r\n");
}

struct HTTPResponse {
    result: String,

    keep_alive: bool,
}


/**
 * Process an http request.
 *
 * Returns a response to be sent.
 * We consume the http request string, watch out for this.
 */
fn process_http_request(s: String) -> HTTPResponse {
    let mut request_url: Option<(String, String)> = None;
    let url_regex = Regex::new(r"^([A-Z]*)\s(.*)\sHTTP/(\d\.\d)").unwrap();

    let mut request_format: Option<String> = None;
    let accept_regex = Regex::new(r"Accept: (.*)").unwrap();

    let mut auth_format: Option<String> = None;
    let auth_regex = Regex::new(r"Authorization: Basic (.*)").unwrap();

    for line in s.split("\n") {
        println!("recv: {}", line);

        // Check if this line has some kind of request url
        if request_url.is_none() {
            if let Some(caps) = url_regex.captures(line) {
                let method = caps.get(1).unwrap().as_str().to_string();
                let url = caps.get(2).unwrap().as_str().to_string();
                let version = caps.get(3).unwrap().as_str();

                println!(
                    "request {} for url {} with http ver {}",
                    method, url, version
                );

                if version != "1.0" && version != "1.1" {
                    return HTTPResponse {
                        result: create_http_response(505, vec![]),
                        keep_alive: false,
                    };
                }

                request_url = Some((method, url))
            }
        }

        // Check the formats the client requested.
        // Only json will be supported, because server metadata is supported that way.
        // Maybe html (and, by extension, CSS?) will be supported one day.
        if request_format.is_none() {
            if let Some(caps) = accept_regex.captures(line) {
                let accept_formats = caps.get(1).unwrap().as_str();

                match accept_formats
                    .split(',')
                    .find(|fmt| fmt.contains("application/json") || fmt.contains("text/html"))
                {
                    Some(fmt) => {
                        request_format = Some(fmt.to_string());
                    }
                    None => {}
                }
            }
        }

        // Check if we have some sort of authentication header.
        // It is not required, except to download maps.
        if auth_format.is_none() {
            if let Some(caps) = auth_regex.captures(line) {
                let auth_data = caps.get(1).unwrap().as_str().to_string();
                auth_format = Some(auth_data)
            }
        }
    }

    if request_url.is_none() {
        return HTTPResponse {
            result: create_http_response(400, vec![]),
            keep_alive: false,
        };
    } else {
        if request_format.is_none() {
            return HTTPResponse {
                result: create_http_response(415, vec![]),
                keep_alive: false,
            };
        }

        let url = request_url.unwrap().1;

        if url == "/info".to_string() {
            let mut result = create_http_response(200, vec![]);

            let server_info =
                ServerInfo::new("Test Server", "Test server description, not in C++", 4);
            match serde_json::to_string(&server_info) {
                Ok(res) => {
                    result.push_str(&res);                    
                    return HTTPResponse {
                        result,
                        keep_alive: false,
                    };
                }
                Err(_) => {
                    return HTTPResponse {
                        result: create_http_response(500, vec![]),
                        keep_alive: false,
                    };
                }
            }
        }

        if url == "/map/08" {
            let result = if auth_format.is_none() {
                create_http_response(
                    401,
                    vec![
                    "WWW-Authenticate: Basic realm=\"Inform the correct key to download this map.\"".to_string()
                ],
                )
            } else {
                // use this mimetype to make browsers download the map instead of
                // showing them in the browser window.
                let mut r = create_http_response(
                    200,
                    vec![
                        "Content-Length: 30".to_string(),
                        "Content-Type: application/octet-stream".to_string(),
                    ],
                );
                r.push_str("FMAPAAAAAAAAAAAAAAAAAAAAAAAAAAA");

                r
            };

            return HTTPResponse {
                result,
                keep_alive: false,
            };
        }

        if url == "/connect".to_string() {
            let mut result = create_http_response(
                101,
                vec![
                    "Connection: upgrade".to_string(),
                    "Upgrade: familyline-server-protocol/0.0.1".to_string(),
                ],
            );

            result.push_str("!FL CONNECT localhost 32000\n\n");

            return HTTPResponse {
                result,
                keep_alive: false,
            };
        }

        return HTTPResponse {
            result: create_http_response(404, vec![]),
            keep_alive: false,
        };
    }
}

async fn process_client(conn: Result<TcpStream, Error>) {
    match conn {
        Err(e) => eprintln!("accept failed = {:?}", e),
        Ok(mut sock) => {
            println!("accept succeeded");

            // Spawn the future that echos the data and returns how
            // many bytes were copied as a concurrent task.
            tokio::spawn(async move {
                // Split up the reading and writing parts of the
                // socket.
                let (mut reader, mut writer) = sock.split();

                loop {
                    let mut buf = vec![0 as u8; 1024];
                    match reader.read(&mut buf).await {
                        Ok(0) => {
                            // Read with length 0 usually means that the client closed the
                            // connection.
                            println!("closed connection");
                            break;
                        }
                        Ok(size) => {
                            let s = match String::from_utf8(buf) {
                                Ok(res) => res,
                                Err(err) => {
                                    eprintln!("error on utf8 conversion: {:?}", err);
                                    continue;
                                }
                            };

                            // Remember that http messages ends with two \r\n

                            let response = process_http_request(s);
                            println!("{}", response.result);

                            match writer.write(&response.result.into_bytes()).await {
                                Ok(_) => println!(">> R {}b>", size),
                                Err(err) => eprintln!("error on write: {:?}", err),
                            }

                            if !response.keep_alive {
                                break;
                            }
                        }
                        Err(err) => eprintln!("error on read: {:?}", err),
                    }
                }

                //match tokio::io::copy(&mut reader, &mut writer).await {
                //    Ok(amt) => {
                //        println!("wrote {} bytes", amt);
                //    }
                //    Err(err) => {
                //        eprintln!("IO error {:?}", err);
                //    }
                //}
            });
        }
    }
}

#[tokio::main]
async fn main() {
    let addr = "127.0.0.1:6142";
    let mut listener = TcpListener::bind(addr).await.unwrap();

    // Here we convert the `TcpListener` to a stream of incoming connections
    // with the `incoming` method.
    let server = async move {
        let mut incoming = listener.incoming();
        while let Some(conn) = incoming.next().await {
            process_client(conn).await;
        }
    };

    println!("Server running on localhost:6142");

    // Start the server and block this async fn until `server` spins down.
    server.await;
}
