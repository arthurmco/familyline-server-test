#[macro_use]
extern crate lazy_static;

use chrono::prelude::*;
use futures::stream::StreamExt;
use regex::Regex;
use tokio::net::{TcpListener, TcpStream, UdpSocket};
use tokio::prelude::*;

use std::io::Error;
use std::net::Ipv4Addr;
use std::sync::{Arc, RwLock};

mod client;
mod server;
use client::{Client, ClientError, ClientResponse};
use server::{ClientInfo, ServerInfo};

use serde::{Deserialize, Serialize};
use serde_json::Value;

struct ServerState {
    info: ServerInfo,
    clients: Vec<Client>,
}

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
        201 => "201 Created",
        400 => "400 Bad Request",
        401 => "401 Unauthorized",
        403 => "403 Forbidden",
        404 => "404 Not Found",
        415 => "415 Unsupported Media Type",
        500 => "500 Internal Server Error",
        503 => "503 Service Unavailable",
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

fn update_client(state: Arc<RwLock<ServerState>>, client_obj: Client) {
    println!("locks: {}", Arc::strong_count(&state));
    let cstate = Arc::clone(&state);
    for i in 0..100 {
        let mut mstate = match cstate.write() {
            Ok(s) => s,
            Err(_) => {
                eprintln!("cannot acquire server state write lock");
                std::thread::sleep(std::time::Duration::from_millis(i * 40));
                continue;
            }
        };

        mstate.clients.push(client_obj);
        let client_infos = mstate.clients.iter().map(|c| ClientInfo::from(c)).collect();
        mstate.info.update_clients(client_infos);
        break;
    }
}

fn remove_client(state: Arc<RwLock<ServerState>>, id: usize) {
    println!("locks: {}", Arc::strong_count(&state));
    let cstate = Arc::clone(&state);
    for i in 0..100 {
        let mut mstate = match cstate.try_write() {
            Ok(s) => s,
            Err(_) => {
                eprintln!("cannot acquire server state write lock");
                std::thread::sleep(std::time::Duration::from_millis(i * 40));
                continue;
            }
        };

        mstate.clients.retain(|c| c.id() != id);
        let client_infos = mstate.clients.iter().map(|c| ClientInfo::from(c)).collect();
        mstate.info.update_clients(client_infos);
        break;
    }
}

fn is_client_list_full(state: &RwLock<ServerState>) -> bool {
    state.read().unwrap().info.is_client_list_full()
}

/// Represents what was found in the http request
struct HTTPRequestInfo {
    // General headers
    url: String,
    method: String,
    format: Option<String>,
    auth: Option<String>,
    length: Option<usize>,

    // Body
    body: Option<String>,

    // Specific headers for the game server
    player_code: Option<String>,
}

fn parse_http_request(s: String) -> Option<HTTPRequestInfo> {
    let mut request_url: Option<(String, String)> = None;
    let url_regex = Regex::new(r"^([A-Z]*)\s(.*)\sHTTP/(\d\.\d)").unwrap();

    let mut request_format: Option<String> = None;
    let accept_regex = Regex::new(r"Accept: (.*)").unwrap();

    let mut auth_format: Option<String> = None;
    let auth_regex = Regex::new(r"Authorization: Basic (.*)").unwrap();

    let mut code_format: Option<String> = None;
    let code_regex = Regex::new(r"X-Client-Code: (.*)").unwrap();

    let mut length_format: Option<usize> = None;
    let length_regex = Regex::new(r"Content-Length: (.*)").unwrap();

    let mut body = None;

    for line in s.split("\n") {
        // An empty line means that the next line is the body
        if line == "\r" || line == "" {
            body = Some("".to_string());
            continue;
        }

        if let Some(s) = body {
            let mut new_s = s.clone();
            let sline = format!("{}\n", line);
            new_s.push_str(&sline);
            body = Some(new_s);
        } else {
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
                        return None;
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
                            request_format = Some(fmt.trim().to_string());
                        }
                        None => {}
                    }
                }
            }

            // Check if we have some sort of authentication header.
            // It is not required, except to download maps.
            if auth_format.is_none() {
                if let Some(caps) = auth_regex.captures(line) {
                    let auth_data = caps.get(1).unwrap().as_str();
                    auth_format = Some(auth_data.trim().to_string())
                }
            }

            // Check for the client code
            // If not here, assume the client did not log on and
            // deny access to all endpoints but the login one
            if code_format.is_none() {
                if let Some(caps) = code_regex.captures(line) {
                    let code_data = caps.get(1).unwrap().as_str();
                    code_format = Some(code_data.trim().to_string())
                }
            }

            // Check for the content length
            //
            // Useful only if we have a body
            if length_format.is_none() {
                if let Some(caps) = length_regex.captures(line) {
                    length_format = match caps.get(1).unwrap().as_str().parse() {
                        Ok(v) => Some(v),
                        Err(_) => None,
                    }
                }
            }
        }
    }

    match request_url {
        None => None,
        Some((method, url)) => Some(HTTPRequestInfo {
            url,
            method,
            format: request_format,
            auth: auth_format,
            length: length_format,
            body: body,
            player_code: code_format,
        }),
    }
}

fn handle_login(state: &Arc<RwLock<ServerState>>, request: &HTTPRequestInfo) -> HTTPResponse {
    if is_client_list_full(&state) {
        let res = "{\"error\": \"CLIENT_LIST_FULL\", \"description\": \"Cannot log, the client list is full\"}";

        let mut result = create_http_response(503, vec![format!("Content-Length: {}", res.len())]);
        result.push_str(&res);
        return HTTPResponse {
            result,
            keep_alive: false,
        };
    }

    match &request.body {
        Some(s) => {
            #[derive(Serialize, Deserialize)]
            struct ClientLoginRequest {
                client_name: String,
            };

            #[derive(Serialize, Deserialize)]
            struct ClientLoginResponse {
                id: usize,
                code: String,
                name: String,
            };

            let request_body = s.trim().trim_matches(char::from(0));

            let client_req: ClientLoginRequest = match serde_json::from_str(request_body) {
                Ok(s) => s,
                Err(_) => {
                    return HTTPResponse {
                        result: create_http_response(400, vec![]),
                        keep_alive: false,
                    };
                }
            };

            let client_obj = Client::new(&client_req.client_name);
            let client_res = ClientLoginResponse {
                id: client_obj.id(),
                code: client_obj.code(),
                name: client_obj.name().to_string(),
            };

            let client_res_str = serde_json::to_string(&client_res).unwrap();

            let mut result = create_http_response(
                201,
                vec![format!("Content-Length: {}", client_res_str.len())],
            );

            result.push_str(&client_res_str);
            update_client(Arc::clone(&state), client_obj);

            println!("{:?}", state.read().unwrap().clients);

            return HTTPResponse {
                result,
                keep_alive: false,
            };
        }
        None => {
            let mut result = create_http_response(401, vec![]);
            result.push_str("{\"error\": \"No body in login request\"}");
            return HTTPResponse {
                result,
                keep_alive: false,
            };
        }
    }
}

/**
 * Process an http request.
 *
 * Returns a response to be sent.
 * We consume the http request string, watch out for this.
 */
fn process_http_request(state: &Arc<RwLock<ServerState>>, s: String) -> HTTPResponse {
    let request = match parse_http_request(s) {
        Some(s) => s,
        None => {
            return HTTPResponse {
                result: create_http_response(400, vec![]),
                keep_alive: false,
            };
        }
    };

    if request.format.is_none() {
        return HTTPResponse {
            result: create_http_response(415, vec![]),
            keep_alive: false,
        };
    }

    match request.player_code {
        None => {
            if request.url == "/login".to_string() {
                return handle_login(&state, &request);
            }
        }
        Some(code) => {
            let client_code = match state.read() {
                Ok(c) => match c.clients.iter().find(|c| c.code() == code) {
                    Some(s) => Some(s.id()),
                    None => None,
                },
                Err(_) => {
                    return HTTPResponse {
                        result: create_http_response(500, vec![]),
                        keep_alive: false,
                    };
                }
            };

            match client_code {
                Some(cid) if request.url.starts_with("/logout") => {
                    eprintln!("removing client id {}", cid);
                    remove_client(Arc::clone(&state), cid);
                    return HTTPResponse {
                        result: create_http_response(200, vec![]),
                        keep_alive: false,
                    };
                }
                Some(cid) => {
                    let rstate = match state.read() {
                        Ok(c) => c,
                        Err(_) => {
                            return HTTPResponse {
                                result: create_http_response(500, vec![]),
                                keep_alive: false,
                            };
                        }
                    };

                    let c = rstate.clients.iter().find(|c| c.id() == cid).unwrap();

                    match c.handle_url(&request.url, &rstate.info) {
                        Ok(res) => {
                            let mut result = create_http_response(200, res.headers);
                            result.push_str(&res.body);
                            return HTTPResponse {
                                result,
                                keep_alive: false,
                            };
                        }
                        Err(e) => {
                            return match e {
                                ClientError::Unauthorized => HTTPResponse {
                                    result: create_http_response(401, vec![]),
                                    keep_alive: false,
                                },
                                ClientError::ResourceNotExist => HTTPResponse {
                                    result: create_http_response(404, vec![]),
                                    keep_alive: false,
                                },
                                ClientError::ServerFailure => HTTPResponse {
                                    result: create_http_response(500, vec![]),
                                    keep_alive: false,
                                },
                                ClientError::UnknownEndpoint => HTTPResponse {
                                    result: create_http_response(404, vec![]),
                                    keep_alive: false,
                                },
                            }
                        }
                    }
                }
                None => {
                    return HTTPResponse {
                        result: create_http_response(403, vec![]),
                        keep_alive: false,
                    }
                }
            }
        }
    }

    return HTTPResponse {
        result: create_http_response(404, vec![]),
        keep_alive: false,
    };
}

async fn process_client(state: &'static Arc<RwLock<ServerState>>, conn: Result<TcpStream, Error>) {
    match conn {
        Err(e) => eprintln!("accept failed = {:?}", e),
        Ok(mut sock) => {
            println!("accept succeeded");

            // Spawn the future that echos the data and returns how
            // many bytes were copied as a concurrent task.
            tokio::spawn(async move {
                // Split up the reading and writing parts of the
                // socket.
                let cstate = Arc::clone(&state);

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
                            let response = process_http_request(&cstate, s);
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

lazy_static! {
    static ref gstate: Arc<RwLock<ServerState>> = Arc::new(RwLock::new(ServerState {
        info: ServerInfo::new("Test Server", "Test server description, not in C++", 4),
        clients: vec![],
    }));
}

// Parse a discover message, return a response to it
fn parse_discover_message(s: String) -> Option<String> {
    enum DiscoverOperation {
        Search,
    };

    let mut op: Option<DiscoverOperation> = None;
    let mut valid_servicetype = false;
    let mut valid_method = false;

    for line in s.split("\r\n") {
        println!("> {}", line);

        if line.starts_with("M-SEARCH *") {
            op = Some(DiscoverOperation::Search);
        }

        if line == "MAN: \"ssdp:discover\"" {
            valid_method = true;
        }

        // Respond to ssdp:all too?
        if line == "ST: game_server:familyline1" {
            valid_servicetype = true;
        }

        // Message ended.
        if line.trim() == "" {
            break;
        }
    }

    if !valid_servicetype || !valid_method {
        return None;
    }

    println!(" -- response is valid --");

    return Some("AAAA".to_string());
}

#[tokio::main]
async fn main() {
    let addr = "127.0.0.1:6142";
    let mut listener = TcpListener::bind(addr).await.unwrap();

    // The main loop for the HTTP part of the protocol
    // Here we convert the `TcpListener` to a stream of incoming connections
    // with the `incoming` method.
    let server = async move {
        let mut incoming = listener.incoming();
        while let Some(conn) = incoming.next().await {
            process_client(&gstate, conn).await;
        }
    };

    println!("Server running on localhost:6142");

    // Add a socket that will be used by clients so they can discover
    // the server. We use UDP so we can simply multicast a search message
    // and then the server will receive it and send the appropriate info
    // to the client.
    // We will use a VERY limited subset of SSDP, basically just the
    // M-SEARCH message. For this reason, we bind the server to another
    // port.
    let mut udp_discover = UdpSocket::bind("0.0.0.0:1983").await.unwrap();
    tokio::spawn(async move {
        udp_discover
            .join_multicast_v4(
                "239.255.255.250".parse().unwrap(),
                "0.0.0.0".parse().unwrap(),
            )
            .unwrap();
        println!("Server is discoverable");
        loop {
            let mut buf = vec![0; 1024];
            match udp_discover.recv_from(&mut buf).await {
                Ok((msize, sockaddr)) => {
                    let msg = match String::from_utf8(buf) {
                        Ok(m) => m,
                        Err(e) => {
                            format!("{:?} ({:?})", e.as_bytes(), e.utf8_error());
                            "".to_string()
                        }
                    };

                    let res = parse_discover_message(msg);
                    if let Some(response) = res {
                        println!("{}", response);
                    }
                }
                Err(_) => {
                    eprintln!("An error happened while receiving a message on the listen socket")
                }
            }
        }
    });

    // Start the server and block this async fn until `server` spins down.
    server.await;
}
