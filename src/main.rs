#[macro_use]
extern crate lazy_static;

use chrono::prelude::*;
use futures::stream::StreamExt;
use tokio::net::{TcpListener, TcpStream, UdpSocket};
use tokio::prelude::*;

use std::io::Error;
use std::net::{Ipv4Addr, SocketAddr};
use std::sync::{Arc, RwLock};

mod client;
mod server;
mod request;
use client::{Client, ClientError, ClientResponse};
use server::{ClientInfo, ServerDiscoveryInfo, ServerInfo};
use request::{HTTPRequestInfo};

use serde::{Deserialize, Serialize};
use serde_json::Value;

struct ServerState {
    info: ServerInfo,
    clients: Vec<Client>,
    host_password: String
}

fn format_rfc2616_date(date: DateTime<Utc>) -> String {
    date.format("%a, %d %b %Y %T GMT").to_string()
}

/// Creates the http response, only the header
fn create_http_response(http_code: u32, content: &str, additional_headers: Vec<String>) -> HTTPResponse {
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

    let body = content.to_string();

    lines.push(format!("HTTP/1.1 {}", codestr));
    lines.push(format!("Date: {}", format_rfc2616_date(now)));
    lines.push("Cache-Control: private, max-age=0".to_string());
    lines.push("Server: familyline-server 0.0.1-test".to_string());

    if body.len() > 0 {
        lines.push(format!("Content-Length: {}", body.len()));
    }

    if http_code >= 200 && http_code <= 299 {
        lines.push("Content-Type: application/json".to_string());
    }

    lines.extend(additional_headers);

    lines.push("".to_string());
    lines.push("".to_string());

    lines.push(body);

    return HTTPResponse {
        result: lines.join("\r\n"),
        keep_alive: false,
    };
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


fn handle_login(state: &Arc<RwLock<ServerState>>, request: &HTTPRequestInfo) -> HTTPResponse {
    if is_client_list_full(&state) {
        let res = "{\"error\": \"CLIENT_LIST_FULL\", \"description\": \"Cannot log, the client list is full\"}";

        return create_http_response(503, &res, vec![]);
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
                    return create_http_response(
                        400,
                        "",
                        vec![],
                    );
                }
            };

            let client_obj = Client::new(&client_req.client_name);
            let client_res = ClientLoginResponse {
                id: client_obj.id(),
                code: client_obj.code(),
                name: client_obj.name().to_string(),
            };

            let client_res_str = serde_json::to_string(&client_res).unwrap();
            update_client(Arc::clone(&state), client_obj);

            println!("{:?}", state.read().unwrap().clients);

            return create_http_response(
                201,
                &client_res_str,
                vec![],
            );
        }
        None => {
            return create_http_response(401, "{\"error\": \"No body in login request\"}", vec![]);
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
    let request = match HTTPRequestInfo::parse_http_request(s) {
        Some(s) => s,
        None => {
            return create_http_response(
                400,
                "",
                vec![],
            );
        }
    };

    if request.format.is_none() {
        return create_http_response(
            415,
            "",
            vec![],
        );
    }

    // We have the following endpoints:
    //  - /login (POST) - log in into the lobby
    //  - /logout (POST)
    //  - /info (GET) - get server information
    //  - /ready (POST) - set the client status as ready to start the game
    //  - /client/<code>/mod (POST) - set/unset moderator status to the specified
    //                                client

    match request.player_code {
        None => {
            if request.url == "/login".to_string() {
                return handle_login(&state, &request);
            }
        }
        Some(code) => {
            // Read the client code first
            // Since we cannot held a read lock while writing, and we
            // write to the state variable in some of the endpoints,
            // we get the client ID, so that we can always find the
            // client, and we do not need to store the client object
            // all times.
            let client_code = match state.read() {
                Ok(c) => match c.clients.iter().find(|c| c.code() == code) {
                    Some(s) => Some(s.id()),
                    None => None,
                },
                Err(_) => {
                    return create_http_response(
                        500,
                        "",
                        vec![],
                    );
                }
            };

            match client_code {
                Some(cid) if request.url.starts_with("/logout") => {
                    eprintln!("removing client id {}", cid);
                    remove_client(Arc::clone(&state), cid);
                    return create_http_response(
                        200,
                        "",
                        vec![],
                    );
                }
                Some(cid) => {
                    let rstate = match state.read() {
                        Ok(c) => c,
                        Err(_) => {
                            return create_http_response(
                                500,
                                "",
                                vec![],
                            );
                        }
                    };

                    let c = rstate.clients.iter().find(|c| c.id() == cid).unwrap();

                    match c.handle_url(&request.url, &rstate.info) {
                        Ok(res) => {
                            return create_http_response(
                                200,
                                &res.body,
                                res.headers,
                            );
                        }
                        Err(e) => {
                            return match e {
                                ClientError::Unauthorized => create_http_response(401, "", vec![]),
                                ClientError::ResourceNotExist => create_http_response(404, "", vec![]),
                                ClientError::ServerFailure => create_http_response(500, "", vec![]),
                                ClientError::UnknownEndpoint => create_http_response(404, "", vec![])
                            }
                        }
                    }
                }
                None => return create_http_response(403, "", vec![])
            }
        }
    }

    return create_http_response(404, "", vec![]);
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
        info: ServerInfo::new("Test Server", "Server written in Rust, not in C++", 4),
        clients: vec![],
        host_password: String::from("123456")
    }));
}

use pnet::datalink;
use vecfold::VecFoldResult;

/// Find local IPv4 address
/// No IPv6 support yet because I am pretty sure my router is shit
/// and will not work well with it.
/// (this is also the reason why I set up only an ipv4 multicast)
///
/// Used primarily for the discover message response we receive in
/// multicast to tell the client where to connect to
///
/// If no address is found, it falls back to the loopback address.
fn find_local_address() -> String {
    for iface in datalink::interfaces()
        .iter()
        .filter(|i| i.ips.len() > 0)
        .filter(|i| {
            if let Some(mac) = i.mac {
                !mac.is_zero()
            } else {
                false
            }
        })
    {
        let ips = &iface.ips;
        return ips
            .iter()
            .filter(|i| i.ip().is_ipv4())
            .map(|i| i.ip().to_string())
            .nth(0)
            .or_else(|| Some(String::from("127.0.0.1")))
            .unwrap();
    }

    return String::from("127.0.0.1");
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

    let sinfo = ServerDiscoveryInfo::from(&gstate.read().unwrap().info);
    let sinfo_str = serde_json::to_string(&sinfo).unwrap();

    let now: DateTime<Utc> = Utc::now();
    let response_lines = vec![
        String::from("HTTP/1.1 200 OK"),
        String::from("Cache-Control: max-age=60"),
        format!("Date: {}", format_rfc2616_date(now)),
        String::from("Ext: "),
        format!("Location: http://{}:6142", find_local_address()),
        format!("Content-Length: {}", sinfo_str.len()),
        String::from("Server: familyline-server 0.0.1-test"),
        String::from(""),
        sinfo_str,
    ];

    println!(" -- response is valid --");

    return Some(response_lines.join("\r\n"));
}

/// Since UDP is unreliable, we might need to send the response
/// packet more than once, because it will not give an error even
/// if the packet did not reach its destination.
async fn send_multiple(
    socket: &mut UdpSocket,
    addr: &SocketAddr,
    buf: String,
    count: usize,
) -> Result<usize, std::io::Error> {
    let data = buf.as_bytes();
    let mut v = vec![];

    for _i in 0..count {
        v.push(socket.send_to(&data, addr).await);
    }

    match v.foldr() {
        Ok(sizes) => Ok(**sizes.iter().max().unwrap()),
        Err(e) => Err(std::io::Error::from_raw_os_error(e.raw_os_error().unwrap())),
    }
}


/**
 * TODO:
 *  - add a way to set the mod player, the player (or players) that can
 *    change game settings before the game starts
 *  - add endpoints to change game settings (map, game type, )
 *  - add support for password-protected servers (add authentication on login)
 *  - change the way map download works: if we use the authentication header for
 *    password-protected servers, we need to find another way to download maps:
 *    a good idea will be send the map anyway, but add support for encryption
 *    in the terrain file itself
 *  - add a basic chat protocol, maybe websockets or BOSH (BOSH seems easier).
 *    We MIGHT use XMPP in the future, especially if we use voice chats, but
 *    for now, a simple chat protocol will be used
 *  - a basic game protocol. We receive the inputs from a single client and send
 *    it to other clients each tick. The in-game protocol will probably be
 *    based on flatbuffers messages.
 */

// TODO: the server host (the admin mod) will be the first one that guesses
// the host password

#[tokio::main]
async fn main() {
    let addr = "0.0.0.0:6142";
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

    println!("Server address is {}", find_local_address());
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
                        match send_multiple(&mut udp_discover, &sockaddr, response.clone(), 2).await
                        {
                            Ok(ssize) => {
                                println!("{:?}, {} bytes\n V \n{}", sockaddr, ssize, response);
                            }
                            Err(e) => {
                                eprintln!("{:?}", e);
                            }
                        }
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
