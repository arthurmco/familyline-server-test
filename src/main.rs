use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::net::SocketAddr;
use warp::{
    http::{Response, StatusCode},
    Filter,
};

use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::sync::oneshot;

mod broadcast;
mod config;
mod messages;

use config::ServerConfiguration;

use messages::QueryError;
use messages::{send_get_client_message, send_info_message, send_login_message};
use messages::{start_message_processor, FMessage, FRequestMessage, FResponseMessage};
use broadcast::run_discovery_thread;
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

fn with_sender(
    sender: Sender<FMessage>,
) -> impl Filter<Extract = (Sender<FMessage>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || sender.clone())
}

/// Setup and run the HTTP login server
async fn run_http_server(config: &ServerConfiguration, sender: Sender<FMessage>) {
    let login = warp::post()
        .and(warp::path("login"))
        .and(warp::body::content_length_limit(1024))
        .and(warp::body::json())
        .and(with_sender(sender.clone()))
        .and_then(serve_login);

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

    let server = warp::serve(login.or(info).or(clients));

    println!("Server started at 127.0.0.1:{}", config.port);

    server.run(([127, 0, 0, 1], config.port)).await;

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
