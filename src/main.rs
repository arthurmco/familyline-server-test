use futures::{FutureExt, StreamExt, TryStreamExt};
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use warp::{
    http::{Response, StatusCode},
    ws::WebSocket,
    Filter,
};

use tokio::sync::oneshot;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, Interest},
    sync::mpsc::{channel, Receiver, Sender},
    time,
};

mod broadcast;
mod client;
mod config;
mod messages;

use config::ServerConfiguration;

use broadcast::{find_local_address, run_discovery_thread};
use messages::QueryError;
use messages::{
    send_connect_message, send_get_client_message, send_info_message, send_login_message,
    send_logout_message, send_set_ready_message, send_unset_ready_message, ConnectInfo,
};
use messages::{start_message_processor, FMessage, FRequestMessage, FResponseMessage};
use std::time::Duration;
/**
 * Our endpoints
 *
 *  - /login [POST]: Get the login token
 *  - /info [POST]:  Get the server info (name, max players, connected players)
 *  - /clients/<id> [POST]: Get more information about a specific client
 *  - /chat [GET]: Load the chat websocket
 *  - /ready/set and /ready/unset [PUT]: Set the ready status of the client
 *
 *  - /start [POST]: Request a match start
 *  - /connect [POST]: Make the client start the match.
 *                     The client start the match when it detects all other clients are ready.
 *
 * We do it on HTTP because it would be easier to use existing
 * protocols than create a new
 * Also, the chat websocket is the only endpoint that works after a game
 * start
 *
 * TODO: how an spectator will connect?
 */

/// Open a request channel
///
/// Both the HTTP login API and the game server will use it
/// to request data.
fn create_request_channel(config: &ServerConfiguration) -> Sender<FMessage> {
    return start_message_processor(config);
}

#[derive(Deserialize, Serialize)]
struct LoginBody {
    name: String,
}

#[derive(Deserialize, Serialize)]
struct AuthBody {
    token: String,
}

#[derive(Deserialize, Serialize)]
struct BasicError {
    message: String,
}

#[derive(Deserialize, Serialize)]
struct LogoutBody {
    id: u64,
}

async fn serve_login(
    body: LoginBody,
    sender: Sender<FMessage>,
) -> Result<impl warp::Reply, Infallible> {
    let mut sender = sender.clone();

    match send_login_message(&mut sender, &body.name).await {
        Ok(login) => Ok(warp::reply::with_status(
            warp::reply::json(&login),
            StatusCode::CREATED,
        )),
        Err(_) => Ok(warp::reply::with_status(
            warp::reply::json(&BasicError {
                message: String::from("Login failure"),
            }),
            StatusCode::BAD_REQUEST,
        )),
    }
}

async fn serve_info(
    body: AuthBody,
    sender: Sender<FMessage>,
) -> Result<impl warp::Reply, Infallible> {
    let mut sender = sender.clone();

    match send_info_message(&mut sender, &body.token).await {
        Ok(sinfo) => Ok(warp::reply::with_status(
            warp::reply::json(&sinfo),
            StatusCode::OK,
        )),
        Err(e) => match e {
            QueryError::InvalidToken => Ok(warp::reply::with_status(
                warp::reply::json(&BasicError {
                    message: String::from("Invalid token"),
                }),
                StatusCode::UNAUTHORIZED,
            )),
            _ => panic!("Unhandled case!!!"),
        },
    }
}

async fn serve_logout(
    body: AuthBody,
    sender: Sender<FMessage>,
) -> Result<impl warp::Reply, Infallible> {
    let mut sender = sender.clone();

    match send_logout_message(&mut sender, &body.token).await {
        Ok(id) => Ok(warp::reply::with_status(
            warp::reply::json(&LogoutBody { id }),
            StatusCode::OK,
        )),
        Err(e) => match e {
            QueryError::InvalidToken => Ok(warp::reply::with_status(
                warp::reply::json(&BasicError {
                    message: String::from("Invalid token"),
                }),
                StatusCode::UNAUTHORIZED,
            )),
            _ => panic!("Unhandled case!!!"),
        },
    }
}

async fn serve_client(
    clientid: u64,
    body: AuthBody,
    sender: Sender<FMessage>,
) -> Result<impl warp::Reply, Infallible> {
    let mut sender = sender.clone();

    match send_get_client_message(&mut sender, &body.token, clientid).await {
        Ok(cinfo) => Ok(warp::reply::with_status(
            warp::reply::json(&cinfo),
            StatusCode::OK,
        )),
        Err(e) => match e {
            QueryError::ClientNotFound => Ok(warp::reply::with_status(
                warp::reply::json(&BasicError {
                    message: String::from("Client not found"),
                }),
                StatusCode::NOT_FOUND,
            )),
            QueryError::InvalidToken => Ok(warp::reply::with_status(
                warp::reply::json(&BasicError {
                    message: String::from("Invalid token"),
                }),
                StatusCode::UNAUTHORIZED,
            )),
            _ => panic!("Unexpected error!"),
        },
    }
}

async fn set_ready(
    body: AuthBody,
    sender: Sender<FMessage>,
) -> Result<impl warp::Reply, Infallible> {
    let mut sender = sender.clone();

    match send_set_ready_message(&mut sender, &body.token).await {
        Ok(v) => match v {
            true => Ok(warp::reply::with_status(
                warp::reply::json(&BasicError {
                    message: String::default(),
                }),
                StatusCode::OK,
            )),
            false => Ok(warp::reply::with_status(
                warp::reply::json(&BasicError {
                    message: String::from("Invalid state change"),
                }),
                StatusCode::INTERNAL_SERVER_ERROR,
            )),
        },
        Err(e) => match e {
            QueryError::ClientNotFound => Ok(warp::reply::with_status(
                warp::reply::json(&BasicError {
                    message: String::from("Client not found"),
                }),
                StatusCode::NOT_FOUND,
            )),
            QueryError::InvalidToken => Ok(warp::reply::with_status(
                warp::reply::json(&BasicError {
                    message: String::from("Invalid token"),
                }),
                StatusCode::UNAUTHORIZED,
            )),
            _ => panic!("Unexpected error!"),
        },
    }
}

async fn serve_connect(
    body: AuthBody,
    sender: Sender<FMessage>,
    config: ServerConfiguration,
) -> Result<impl warp::Reply, Infallible> {
    let mut sender = sender.clone();

    match send_connect_message(&mut sender, &body.token).await {
        Ok(v) => Ok(warp::reply::with_status(
            warp::reply::json(&ConnectInfo {
                address: find_local_address(),
                port: config.gameport,
            }),
            StatusCode::OK,
        )),
        Err(e) => match e {
            QueryError::ClientNotFound => Ok(warp::reply::with_status(
                warp::reply::json(&BasicError {
                    message: String::from("Client not found"),
                }),
                StatusCode::NOT_FOUND,
            )),
            QueryError::NotAllClientsReady => Ok(warp::reply::with_status(
                warp::reply::json(&BasicError {
                    message: String::from("Not all clients are ready!"),
                }),
                StatusCode::BAD_REQUEST,
            )),
            QueryError::InvalidToken => Ok(warp::reply::with_status(
                warp::reply::json(&BasicError {
                    message: String::from("Invalid token"),
                }),
                StatusCode::UNAUTHORIZED,
            )),
        },
    }
}

async fn unset_ready(
    body: AuthBody,
    sender: Sender<FMessage>,
) -> Result<impl warp::Reply, Infallible> {
    let mut sender = sender.clone();

    match send_unset_ready_message(&mut sender, &body.token).await {
        Ok(v) => match v {
            true => Ok(warp::reply::with_status(
                warp::reply::json(&BasicError {
                    message: String::default(),
                }),
                StatusCode::OK,
            )),
            false => Ok(warp::reply::with_status(
                warp::reply::json(&BasicError {
                    message: String::from("Invalid state change"),
                }),
                StatusCode::INTERNAL_SERVER_ERROR,
            )),
        },
        Err(e) => match e {
            QueryError::ClientNotFound => Ok(warp::reply::with_status(
                warp::reply::json(&BasicError {
                    message: String::from("Client not found"),
                }),
                StatusCode::NOT_FOUND,
            )),
            QueryError::InvalidToken => Ok(warp::reply::with_status(
                warp::reply::json(&BasicError {
                    message: String::from("Invalid token"),
                }),
                StatusCode::UNAUTHORIZED,
            )),
            _ => panic!("Unexpected error!"),
        },
    }
}

fn with_sender(
    sender: Sender<FMessage>,
) -> impl Filter<Extract = (Sender<FMessage>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || sender.clone())
}

fn with_config(
    config: ServerConfiguration,
) -> impl Filter<Extract = (ServerConfiguration,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || config.clone())
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ChatMessageBody {
    receiver: String,
    content: String,
}

async fn send_chat_message(c: ChatMessageBody, token: &str) {}

async fn handle_chat(ws: WebSocket, sender: Sender<FMessage>) {
    let (mut tx, mut rx) = ws.split();
    let mut token: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
    println!("???");

    // We need to send messages in a separate thread because, for some reason,
    // warp and Rust streams has no easy way to get a non-blocking read
    let stoken = Arc::clone(&token);
    tokio::spawn(async move {
        loop {
            let token = Arc::clone(&stoken);
            if token.lock().unwrap().is_some() {
                println!("sending messages");
                tokio::time::sleep(Duration::from_millis(900)).await
            }
        }
    });

    loop {
        let token = Arc::clone(&token);

        match rx.try_next().await {
            Ok(result) => match result {
                Some(result) => {
                    // First, we authenticate
                    //
                    // The client needs to send its token to the server
                    if token.lock().unwrap().is_none() {
                        let msg = result;
                        let msgstr = msg.to_str().unwrap_or_default();
                        let auth: Option<AuthBody> = serde_json::from_str(msgstr).ok();
                        *token.lock().unwrap() = match auth {
                            Some(auth) => Some(auth.token),
                            None => None,
                        };
                    } else {
                        let utoken = token.lock().unwrap();
                        let vtoken = match *utoken {
                            Some(ref v) => v.clone(),
                            None => String::default(),
                        };

                        let msg = result;
                        match msg.to_str() {
                            Ok(msg) => {
                                let cmsg: Option<ChatMessageBody> = serde_json::from_str(msg).ok();
                                match cmsg {
                                    Some(cmsg) => send_chat_message(cmsg, &vtoken),
                                    None => continue,
                                }
                            }
                            Err(_) => continue,
                        };
                    }
                }
                None => {
                    println!("e");
                }
            },
            Err(_) => {
                println!("error?");
                /// no message will be received
                ()
            }
        }
    }
}

/// Setup and run the HTTP login server
async fn run_http_server(config: &ServerConfiguration, sender: Sender<FMessage>) {
    let login = warp::post()
        .and(warp::path("login"))
        .and(warp::body::content_length_limit(1024))
        .and(warp::body::json())
        .and(with_sender(sender.clone()))
        .and_then(serve_login);

    let logout = warp::post()
        .and(warp::path("logout"))
        .and(warp::body::content_length_limit(1024))
        .and(warp::body::json())
        .and(with_sender(sender.clone()))
        .and_then(serve_logout);

    let info = warp::post()
        .and(warp::path("info"))
        .and(warp::body::content_length_limit(1024))
        .and(warp::body::json())
        .and(with_sender(sender.clone()))
        .and_then(serve_info);

    let clients = warp::post()
        .and(warp::path!("clients" / u64))
        .and(warp::body::content_length_limit(1024))
        .and(warp::body::json())
        .and(with_sender(sender.clone()))
        .and_then(serve_client);

    let connect = warp::post()
        .and(warp::path!("connect"))
        .and(warp::body::content_length_limit(1024))
        .and(warp::body::json())
        .and(with_sender(sender.clone()))
        .and(with_config(config.clone()))
        .and_then(serve_connect);

    let setready = warp::put()
        .and(warp::path("set"))
        .and(warp::body::json())
        .and(with_sender(sender.clone()))
        .and_then(set_ready);

    let unsetready = warp::put()
        .and(warp::path("unset"))
        .and(warp::body::content_length_limit(1024))
        .and(warp::body::json())
        .and(with_sender(sender.clone()))
        .and_then(unset_ready);

    let ready = warp::path("ready")
        .and(warp::body::content_length_limit(1024))
        .and(setready.or(unsetready));

    let chat = warp::path("chat")
        .and(with_sender(sender.clone()))
        .and(warp::ws())
        .map(|sender: Sender<FMessage>, ws: warp::ws::Ws| {
            ws.on_upgrade(move |websocket| handle_chat(websocket, sender.clone()))
        });

    let server = warp::serve(
        login
            .or(info)
            .or(logout)
            .or(clients)
            .or(chat)
            .or(connect)
            .or(ready),
    );

    println!("Server started at 127.0.0.1:{}", config.port);

    server.run(([0, 0, 0, 0], config.port)).await;

    println!("Server is shutting down")
}

use tokio::net::{TcpListener, TcpStream, UdpSocket};

extern crate flatbuffers;

#[allow(dead_code, unused_imports)]
#[path = "./network_generated.rs"]
mod network_generated;
pub use network_generated::{
    get_root_as_net_packet, Message, NetPacket, StartRequest, StartResponse,
};

/// Create the network packet
///
/// It goes like this:
///
/// | Field        | Size | Description                               |
/// | ------------ | ---- | ----------------------------------------- |
/// | magic        | 4    | The magic header for the message          |
/// | flags        | 4    | Some flags for the message. Currently 0   |
/// | checksum     | 4    | The CRC32 checksum of the whole message   |
/// | payloadsize  | 4    | Size of the payload, in bytes             |
/// | payload      | n    | The payload                               |
///
/// Returns a vector with the binary data for the packet
pub fn create_packet(packet: &NetPacket, builder: &flatbuffers::FlatBufferBuilder) -> Vec<u8> {
    let magic = "FAMI";
    // builder.finish(packet, None);
    let payload = builder.finished_data();
    let psize = payload.len();
    let mut res = Vec::new();

    res.extend(&magic.as_bytes().to_vec());
    res.extend(&vec![0, 0, 0, 0]); // flags
    res.extend(&vec![0, 0, 0, 0]); // checksum (to be filled later)
    res.extend(&vec![
        psize as u8 & 0xff,
        (psize >> 8) as u8 & 0xff,
        (psize >> 18) as u8 & 0xff,
        0
    ]); // checksum (to be filled later)
    res.extend(payload);

    res
}

/// Decode the network packet
pub fn decode_packet(packet: &[u8]) -> Option<NetPacket> {
    if packet.len() <= 16 {
        return None;
    }

    let magic = match String::from_utf8(packet[0..4].to_vec()) {
        Ok(s) => {
            if s != "FAMI" {
                return None;
            } else {
                s
            }
        }
        Err(_) => return None, // Invalid message
    };
    let flags = packet[7] as u32 | ((packet[6] as u32) << 8) | ((packet[5] as u32) << 16);
    if flags != 0 {
        return None;
    }

    let checksum = packet[8] as u32
        | ((packet[9] as u32) << 8)
        | ((packet[10] as u32) << 16)
        | ((packet[11] as u32) << 24);
    if checksum == 0 {
        return None;
    }

    let psize = (packet[12] as u32
        | ((packet[13] as u32) << 8)
        | ((packet[14] as u32) << 16)
        | ((packet[15] as u32) << 24)) as usize;

    if psize > packet.len() {
        return None;
    }

    let payload = &packet[16..(psize - 16)];

    Some(get_root_as_net_packet(&payload))
}

/// Run the game server thread
///
/// Even if you connect, the game server will only accept your messages and answer them if
/// you call /connect and receive a positive answer
pub async fn run_game_server_thread(config: &ServerConfiguration, sender: Sender<FMessage>) {
    let listener = TcpListener::bind(format!("0.0.0.0:{}", config.gameport)).await;
    let mut listener = listener.unwrap();

    println!("Game server started at port {}", config.gameport);
    tokio::spawn(async move {
        while let Ok((mut socket, peer)) = listener.accept().await {
            tokio::spawn(async move {
                println!("Client Connected from: {}", peer.to_string());

                let mut size = [0u8; 1024];
                let mut valid = true;
                while valid {
                    let ready = socket
                        .ready(Interest::READABLE | Interest::WRITABLE)
                        .await
                        .unwrap();

                    if ready.is_readable() {
                        let data = match socket.read(&mut size[..]).await {
                            Ok(s) => {
                                if s == 0 {
                                    valid = false;
                                }
                                size[0..s].to_vec()
                            }
                            Err(e) => {
                                eprintln!("Error while reading: {:?}", e);
                                valid = false;
                                vec![0]
                            }
                        };
       
                        if !valid {
                            break;
                        }

                        let packet = decode_packet(&data[..]);
                        println!("{:?}", packet);
                    }

                    if ready.is_writable() {
                        let str = String::from("QUE BUNITA");
                        match socket.write(str.as_bytes()).await {
                            Ok(s) => {
                                if s <= 0 { valid = false; }
                                ()
                            }
                            Err(e) => {
                                eprintln!("Error while writing: {:?}", e);
                                valid = false;
                                ()
                            }
                        }

                        tokio::time::sleep(Duration::from_millis(1000)).await;
                    }

                }
                println!("Client disconnected from: {}", peer.to_string());
            });
        }
    });
}

#[tokio::main]
async fn main() {
    println!("Starting Familyline server");

    let config = ServerConfiguration::load();

    let sender = create_request_channel(&config);
    run_discovery_thread(&config, sender.clone()).await;
    run_game_server_thread(&config, sender.clone()).await;
    run_http_server(&config, sender.clone()).await;
}
