use std::net::SocketAddr;

use warp::{
    http::{Response, StatusCode},
    Filter,
};


struct ServerConfiguration {
    port: u16,
}

impl ServerConfiguration {
    pub fn load() -> ServerConfiguration {
        ServerConfiguration { port: 8100 }
    }
}

///////////

enum ChatReceiver {
    All,
    Team(u32),
    Client(u64)
}

struct ChatMessage {
    sender_id: u64,
    content: String
}

/////////

enum RequestMessage {
    Login(u64),
    GetServerInfo,
    GetClientInfo(u64),
    
    SendMessage(ChatReceiver, String),
    ReceiveMessages(Vec<ChatMessage>)
}
    
//////////

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


async fn run_http_server(config: &ServerConfiguration) {

    let login = warp::post()
        .and(warp::path("login"))
        .map(|| "Logging in");
        

    let test = warp::get()
        .and(warp::path("test"))
        .map(|| "Hello world!");

    let clients = warp::get()
        .and(warp::path!("clients" / u64)).map(
            |cid| format!("Getting client {}", cid));

        
    
    let server = warp::serve(login.or(test).or(clients));

    println!("Server started at 127.0.0.1:{}", config.port);

    server.run(([127, 0, 0, 1], config.port)).await;

    println!("Server is shutting down")
}

#[tokio::main]
async fn main() {
    println!("Starting Familyline server");

    let config = ServerConfiguration::load();

    run_http_server(&config).await;
}
