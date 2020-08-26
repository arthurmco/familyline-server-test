#[macro_use]
extern crate lazy_static;

extern crate base64;
extern crate sha1;

use futures::stream::StreamExt;
use tokio::net::{TcpListener, TcpStream, UdpSocket};
use tokio::prelude::*;

use std::collections::HashMap;
use std::collections::VecDeque;
use std::io::Error;
use std::net::{Ipv4Addr, SocketAddr};
use std::sync::{Arc, RwLock};

mod chat;
mod client;
mod request;
mod server;
mod state;
use chat::{handle_chat_conversation, send_pending_chat_messages, ChatMessage};
use client::{Client, ClientError, ClientResponse};
use request::{HTTPRequestInfo, HTTPResponse};
use server::{ServerDiscoveryInfo, ServerInfo};
use state::ServerState;

use serde::{Deserialize, Serialize};
use serde_json::Value;

fn modify_client<F, R>(state: Arc<RwLock<ServerState>>, client_id: usize, modify_fn: F) -> Option<R>
where
    F: FnOnce(&mut Client, &ServerState) -> R,
{
    let cstate = Arc::clone(&state);
    let mut mstate = match cstate.write() {
        Ok(s) => s,
        Err(_) => {
            eprintln!("cannot acquire server state write lock");
            return None;
        }
    };

    let (idx, res, new_client) = match mstate
        .info
        .get_clients()
        .iter()
        .enumerate()
        .find(|(i, c)| c.id() == client_id)
    {
        Some((i, c)) => {
            let mut cl = c.clone();
            let res = modify_fn(&mut cl, &mstate);
            (i, Some(res), Some(cl))
        }
        None => (0, None, None),
    };

    match new_client {
        Some(nc) => {
            mstate.info.update_client(idx as usize, nc);
            return res;
        }
        None => {}
    }

    return None;
}

fn update_client(state: Arc<RwLock<ServerState>>, client_obj: Client) {
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

        mstate.info.add_client(client_obj);
        break;
    }
}

fn remove_client(state: Arc<RwLock<ServerState>>, id: usize) {
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

        mstate.info.remove_client(id);
        break;
    }
}

fn is_client_list_full(state: &RwLock<ServerState>) -> bool {
    state.read().unwrap().info.is_client_list_full()
}

fn handle_login(state: &Arc<RwLock<ServerState>>, request: &HTTPRequestInfo) -> HTTPResponse {
    if is_client_list_full(&state) {
        let res = "{\"error\": \"CLIENT_LIST_FULL\", \"description\": \"Cannot log, the client list is full\"}";

        return HTTPResponse::new(503, &res, vec![], &request.url);
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
                    return HTTPResponse::new(400, "", vec![], &request.url);
                }
            };

            let client_obj = Client::new(&client_req.client_name);
            let client_res = ClientLoginResponse {
                id: client_obj.id(),
                code: client_obj.code(),
                name: client_obj.name().to_string(),
            };

            let client_res_str = serde_json::to_string(&client_res).unwrap();
            update_client(Arc::clone(state), client_obj);

            return HTTPResponse::new(201, &client_res_str, vec![], &request.url);
        }
        None => {
            return HTTPResponse::new(
                401,
                "{\"error\": \"No body in login request\"}",
                vec![],
                &request.url,
            );
        }
    }
}

/// Check if the websocket headers are valid
///
/// Return the websocket key (in the field Sec-WebSocket-Key), or none
fn has_websocket_headers(headers: &Vec<(String, String)>) -> Option<String> {
    let mut has_ws = false;
    let mut has_conn = false;
    let mut ws_ver: Option<String> = None;
    let mut ws_key: Option<String> = None;

    for (h, v) in headers {
        if h == "Upgrade" && v == "websocket" {
            has_ws = true
        }

        if h == "Connection" && v == "Upgrade" {
            has_conn = true
        }

        if h == "Sec-WebSocket-Version" {
            ws_ver = Some(String::from(v))
        }

        if h == "Sec-WebSocket-Key" {
            ws_key = Some(String::from(v))
        }
    }

    if !has_ws
        || !has_conn
        || ws_ver.is_none()
        || ws_ver != Some(String::from("13"))
        || ws_key.is_none()
    {
        return None;
    }

    return ws_key;
}

/// Handle the initial handshake, plus the responses of the chat
///
/// The chat is websocket-based, so the thread will usually spin on this function.
fn handle_chat(state: &Arc<RwLock<ServerState>>, request: &HTTPRequestInfo) -> HTTPResponse {
    // Check if we have the required headers
    let wskey = match has_websocket_headers(&request.other_headers) {
        Some(w) => w,
        None => return HTTPResponse::new(400, "", vec![], &request.url),
    };

    let mut m = sha1::Sha1::new();
    let mut retkey = format!("{}{}", wskey, "258EAFA5-E914-47DA-95CA-C5AB0DC85B11");
    m.update(retkey.as_bytes());
    let serverkey = base64::encode(m.digest().bytes());

    let mut r = HTTPResponse::new(
        101,
        "",
        vec![
            String::from("Upgrade: websocket"),
            String::from("Connection: Upgrade"),
            format!("Sec-Websocket-Accept: {}", serverkey),
        ],
        &request.url,
    );

    r.is_chat_endpoint = true;
    return r;
}

fn parse_url_handle_result(cres: Result<ClientResponse, ClientError>, url: &str) -> HTTPResponse {
    match cres {
        Ok(res) => {
            return HTTPResponse::new(res.response_code as u32, &res.body, res.headers, url);
        }
        Err(e) => {
            return match e {
                ClientError::BadInput => HTTPResponse::new(400, "", vec![], url),
                ClientError::Unauthorized => HTTPResponse::new(401, "", vec![], url),
                ClientError::ResourceNotExist => HTTPResponse::new(404, "", vec![], url),
                ClientError::ServerFailure => HTTPResponse::new(500, "", vec![], url),
                ClientError::UnknownEndpoint => HTTPResponse::new(404, "", vec![], url),
            }
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
            return HTTPResponse::new(400, "", vec![], "/");
        }
    };

    if request.format.is_none() && (request.url != "/chat" && request.url != "/login") {
        return HTTPResponse::new(415, "", vec![], &request.url);
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

            if request.url == "/chat".to_string() {
                return handle_chat(&state, &request);
            }
        }
        Some(code) => {
            // Read the client code first
            // Since we cannot held a read lock while writing, and we
            // write to the state variable in some of the endpoints,
            // we get the client ID, so that we can always find the
            // client, and we do not need to store the client object
            // all times.
            let (client_code, is_url_mutable) = match state.read() {
                Ok(c) => match c.info.get_clients().iter().find(|c| c.code() == code) {
                    Some(s) => (Some(s.id()), s.is_url_mutable(&request.url)),
                    None => (None, false),
                },
                Err(_) => {
                    return HTTPResponse::new(500, "", vec![], &request.url);
                }
            };

            match client_code {
                Some(cid) if request.url.starts_with("/logout") => {
                    eprintln!("removing client id {}", cid);
                    remove_client(Arc::clone(&state), cid);
                    return HTTPResponse::new(200, "", vec![], &request.url);
                }
                Some(cid) if is_url_mutable == true => {
                    let rbody = if let Some(b) = request.body {
                        b
                    } else {
                        String::from("")
                    };

                    return parse_url_handle_result(
                        {
                            let rurl = request.url.clone();

                            modify_client(Arc::clone(&state), cid, |mut c, rstate| {
                                c.handle_url_mut(&rurl, &rstate.info, &rbody)
                            })
                            .unwrap()
                        },
                        &request.url,
                    );
                }
                Some(cid) => {
                    let rstate = match state.read() {
                        Ok(c) => c,
                        Err(_) => {
                            return HTTPResponse::new(500, "", vec![], &request.url);
                        }
                    };

                    let c = rstate
                        .info
                        .get_clients()
                        .iter()
                        .find(|c| c.id() == cid)
                        .unwrap();

                    let rbody = if let Some(b) = request.body {
                        b
                    } else {
                        String::from("")
                    };
                    return parse_url_handle_result(
                        c.handle_url(&request.url, &rstate.info, &rbody),
                        &request.url,
                    );
                }
                None => return HTTPResponse::new(403, "", vec![], &request.url),
            }
        }
    }

    return HTTPResponse::new(404, "", vec![], &request.url);
}

async fn process_client(state: &'static Arc<RwLock<ServerState>>, conn: Result<TcpStream, Error>) {
    match conn {
        Err(e) => eprintln!("accept failed = {:?}", e),
        Ok(mut sock) => {
            let source_ip = sock.peer_addr().unwrap().ip().to_string();

            // Spawn the future that echos the data and returns how
            // many bytes were copied as a concurrent task.
            tokio::spawn(async move {
                // Split up the reading and writing parts of the
                // socket.
                let cstate = Arc::clone(&state);

                let (mut reader, mut writer) = sock.split();
                let mut is_chat_conversation = false;
                let client_id = 0;

                loop {
                    if is_chat_conversation {
                        let ret = send_pending_chat_messages(&cstate, client_id);
                        if ret.len() > 0 {
                            writer.write(&ret).await;
                        }
                    }

                    let mut buf = vec![0 as u8; 4096];
                    match reader.read(&mut buf).await {
                        Ok(0) => {
                            // Read with length 0 usually means that the client closed the
                            // connection.
                            break;
                        }
                        Ok(size) => {
                            if is_chat_conversation {
                                let ret = handle_chat_conversation(&cstate, &buf, client_id);

                                if ret.len() > 0 {
                                    writer.write(&ret).await;
                                }

                                continue;
                            }

                            let s = match String::from_utf8(buf) {
                                Ok(res) => res,
                                Err(err) => {
                                    eprintln!("error on utf8 conversion: {:?}", err);
                                    continue;
                                }
                            };

                            // Remember that http messages ends with two \r\n
                            let response = process_http_request(&cstate, s);

                            match writer.write(&response.result.into_bytes()).await {
                                Ok(_) => {
                                    let sdate =
                                        response.response_date.format("%d/%b/%Y:%H:%M:%S %z");
                                    println!(
                                        "{} - - [{}] \"GET {} HTTP/1.1\" {} {}",
                                        source_ip,
                                        sdate,
                                        response.request_url,
                                        response.http_code,
                                        response.body_size
                                    )
                                }
                                Err(err) => eprintln!("error on write: {:?}", err),
                            }

                            if !response.is_chat_endpoint {
                                break;
                            } else {
                                eprintln!("<client {} opened the chat websocket", source_ip);
                                is_chat_conversation = true;
                            }
                        }
                        Err(err) => eprintln!("error on read: {:?}", err),
                    }
                }
            });
        }
    }
}

/// Basic server structure
///
/// Usually things that are static for the server
pub struct ServerSettings {
    name: String,
    version: String,
    description: String,
    host_password: String,
}

impl ServerSettings {
    pub fn get() -> ServerSettings {
        ServerSettings {
            name: String::from("Test Server"),
            version: String::from("0.1.99"),
            description: String::from("Server written in Rust, not in C++"),
            host_password: String::from("123456"),
        }
    }
}

lazy_static! {
    static ref gstate: Arc<RwLock<ServerState>> = Arc::new(RwLock::new(ServerState {
        info: ServerInfo::new(
            "Test Server",
            "Server written in Rust, not in C++",
            4,
            "123456"
        ),
        chats: HashMap::new(),
    }));
}

use vecfold::VecFoldResult;

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

    println!("Server address is {}", request::find_local_address());
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

                    let res = request::parse_discover_message(
                        msg,
                        &ServerDiscoveryInfo::from(&gstate.read().unwrap().info),
                    );
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
