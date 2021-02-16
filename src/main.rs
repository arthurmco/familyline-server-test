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
    sync::mpsc::{channel, Receiver, Sender},
    time,
};

mod broadcast;
mod config;
mod messages;

use config::ServerConfiguration;

use broadcast::run_discovery_thread;
use messages::QueryError;
use messages::{
    send_get_client_message, send_info_message, send_login_message, send_logout_message,
};
use messages::{send_set_ready_message, send_unset_ready_message};
use messages::{start_message_processor, FMessage, FRequestMessage, FResponseMessage};
use std::time::Duration;
/**
 * Our endpoints
 *
 *  - /login [POST]: Get the login token
 *  - /info [GET]:  Get the server info (name, max players, connected players)
 *  - /clients/<id> [GET]: Get more information about a specific client
 *  - /chat [GET]: Load the chat websocket
 *
 *  - /start [POST]: Request a match start
 *  - /connect [POST]: Start the match, and forward the client to the
 *                     port of the game server itself
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
            true => Ok(
                warp::reply::with_status(warp::reply::json(&BasicError {
                    message: String::default(),
                }),
                StatusCode::OK),
            ),
            false => Ok(
                warp::reply::with_status(warp::reply::json(&BasicError {
                    message: String::from("Invalid state change"),
                }),
                StatusCode::INTERNAL_SERVER_ERROR),
            ),
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
            true => Ok(
                warp::reply::with_status(warp::reply::json(&BasicError {
                    message: String::default(),
                }),
                StatusCode::OK)
            ),
            false => Ok(
                warp::reply::with_status(warp::reply::json(&BasicError {
                    message: String::from("Invalid state change"),
                }),
                StatusCode::INTERNAL_SERVER_ERROR)
            ),
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
        },
    }
}

fn with_sender(
    sender: Sender<FMessage>,
) -> impl Filter<Extract = (Sender<FMessage>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || sender.clone())
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
                tokio::time::delay_for(Duration::from_millis(900)).await
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
            .or(ready),
    );

    println!("Server started at 127.0.0.1:{}", config.port);

    server.run(([0, 0, 0, 0], config.port)).await;

    println!("Server is shutting down")
}

#[tokio::main]
async fn main() {
    println!("Starting Familyline server");

    let config = ServerConfiguration::load();

    let sender = create_request_channel(&config);
    run_discovery_thread(&config, sender.clone()).await;
    run_http_server(&config, sender).await;
}
