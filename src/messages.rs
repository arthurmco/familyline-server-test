use rand::Rng;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::sync::oneshot;

use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

///////

struct FClient {
    name: String,
    id: u64,
    token: String,
}

impl FClient {
    fn new(name: String) -> FClient {
        let mut rng = rand::thread_rng();

        let id = rng.gen();
        let tokenbase = format!("{}%{}", id, name);

        let mut s = DefaultHasher::new();
        tokenbase.hash(&mut s);
        let token = format!("{:016x}", s.finish());

        FClient { name, id, token }
    }
}

struct FServer {
    clients: Vec<FClient>,
    name: String,
}

impl FServer {
    fn new(_config: &ServerConfiguration) -> FServer {
        FServer {
            clients: vec![],
            name: format!("Test server"),
        }
    }

    /// Add a client to the server database, returns a reference to it
    fn add_client(&mut self, c: FClient) -> &FClient {
        self.clients.push(c);
        self.clients.last().unwrap()
    }

    /// Check what user do this token belongs
    ///
    /// If it belongs to some user, return it
    /// Else, returns None
    fn validate_token(&self, token: &str) -> Option<&FClient> {
        self.clients.iter().find(|c| c.token == token)
    }
}

////////

#[derive(Debug)]
pub enum ChatReceiver {
    All,
    Team(u32),
    Client(u64),
}

#[derive(Debug)]
pub struct ChatMessage {
    sender_id: u64,
    content: String,
}

///////////

/// Errors that can happen if you try to login and
/// generate the token
pub enum LoginError {}

/// Errors that can happen if you try to authenticate
/// with your token
pub enum ValidateLoginError {
    InvalidToken,
}

#[derive(Debug)]
pub struct LoginInfo {
    token: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LoginResult {
    name: String,
    token: String,
    user_id: u64,
}

impl From<&FClient> for LoginResult {
    fn from(c: &FClient) -> Self {
        LoginResult {
            name: c.name.clone(),
            token: c.token.clone(),
            user_id: c.id,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ClientInfo {
    name: String,
    user_id: u64,
}

impl From<&FClient> for ClientInfo {
    fn from(c: &FClient) -> Self {
        ClientInfo {
            name: c.name.clone(),
            user_id: c.id,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ServerInfo {
    name: String,
    clients: Vec<ClientInfo>,
    max_clients: usize,
}

impl From<&FServer> for ServerInfo {
    fn from(s: &FServer) -> Self {
        ServerInfo {
            name: s.name.clone(),
            clients: s.clients.iter().map(|c| ClientInfo::from(c)).collect(),
            max_clients: 8,
        }
    }
}

#[derive(Debug)]
pub enum FRequestMessage {
    Login(String),
    ValidateLogin(LoginInfo),

    GetServerInfo,
    GetClientInfo(u64),

    SendMessage(ChatReceiver, String),
    ReceiveMessages(Vec<ChatMessage>),
}

pub enum FResponseMessage {
    Login(Result<LoginResult, LoginError>),
    ValidateLogin(Result<LoginResult, ValidateLoginError>),

    GetServerInfo(ServerInfo),
}

pub type FMessage = (FRequestMessage, oneshot::Sender<FResponseMessage>);

////////////

#[derive(Copy, Clone)]
pub struct ServerConfiguration {
    pub port: u16,
}

impl ServerConfiguration {
    pub fn load() -> ServerConfiguration {
        ServerConfiguration { port: 8100 }
    }
}

///////////

/// Open a request channel
///
/// Both the HTTP login API and the game server will use it
/// to request data.
pub fn start_message_processor(config: &ServerConfiguration) -> Sender<FMessage> {
    let (tx, mut rx): (Sender<FMessage>, Receiver<FMessage>) = channel(100);

    let config = config.clone();
    tokio::spawn(async move {
        let mut server = FServer::new(&config);

        while let Some((cmd, response)) = rx.recv().await {
            println!("{:?}", cmd);
            match cmd {
                FRequestMessage::Login(user) => {
                    let client = server.add_client(FClient::new(user));

                    response.send(FResponseMessage::Login(Ok(LoginResult::from(client))));
                }

                FRequestMessage::ValidateLogin(info) => match server.validate_token(&info.token) {
                    Some(client) => {
                        response.send(FResponseMessage::ValidateLogin(Ok(LoginResult::from(
                            client,
                        ))));
                    }
                    None => {
                        response.send(FResponseMessage::ValidateLogin(Err(
                            ValidateLoginError::InvalidToken,
                        )));
                    }
                },
                FRequestMessage::GetServerInfo => {
                    response.send(FResponseMessage::GetServerInfo(ServerInfo::from(&server)));
                }
                _ => panic!("Unsupported message {:?}", cmd),
            };
        }
    });

    return tx;
}

pub async fn send_login_message(
    sender: &mut Sender<FMessage>,
    username: &str,
) -> Result<LoginResult, LoginError> {
    let (resp_tx, resp_rx) = oneshot::channel();

    sender
        .send((FRequestMessage::Login(String::from(username)), resp_tx))
        .await
        .ok()
        .unwrap();
    let res = resp_rx.await.unwrap();

    if let FResponseMessage::Login(res) = res {
        res
    } else {
        panic!("Unexpected response while trying to login");
    }
}

pub async fn send_validate_login_message(
    sender: &mut Sender<FMessage>,
    token: &str,
) -> Result<LoginResult, ValidateLoginError> {
    let (resp_tx, resp_rx) = oneshot::channel();

    sender
        .send((
            FRequestMessage::ValidateLogin(LoginInfo {
                token: String::from(token),
            }),
            resp_tx,
        ))
        .await
        .ok()
        .unwrap();
    let res = resp_rx.await.unwrap();

    if let FResponseMessage::ValidateLogin(res) = res {
        res
    } else {
        panic!("Unexpected response while trying to login");
    }
}

pub async fn send_info_message(
    sender: &mut Sender<FMessage>,
    token: &str,
) -> Result<ServerInfo, ValidateLoginError> {
    match send_validate_login_message(sender, &token).await {
        Ok(login) => {
            let (resp_tx, resp_rx) = oneshot::channel();

            sender
                .send((FRequestMessage::GetServerInfo, resp_tx))
                .await
                .ok()
                .unwrap();
            let res = resp_rx.await.unwrap();

            if let FResponseMessage::GetServerInfo(res) = res {
                Ok(res)
            } else {
                panic!("Unexpected response while trying to login");
            }
        }
        Err(e) => Err(e),
    }
}
