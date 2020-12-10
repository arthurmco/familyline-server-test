use rand::Rng;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::sync::oneshot;

use crate::config::ServerConfiguration;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::collections::VecDeque;
use std::hash::{Hash, Hasher};

use chrono::{offset::Utc, DateTime};

///////

#[derive(Debug, Clone)]
pub enum ChatReceiver {
    All,
    Team,
    Client(u64),
}

#[derive(Debug, Clone)]
pub struct ChatMessage {
    sender_id: u64,
    receiver: ChatReceiver,
    content: String,
    store_date: DateTime<Utc>,
}

struct FClient {
    name: String,
    id: u64,
    token: String,

    last_receive_sent_request: DateTime<Utc>,
    send_queue: VecDeque<ChatMessage>,
    receive_queue: VecDeque<ChatMessage>,
}

impl FClient {
    fn new(name: String) -> FClient {
        let mut rng = rand::thread_rng();

        let id = rng.gen();
        let tokenbase = format!("{}%{}", id, name);

        let mut s = DefaultHasher::new();
        tokenbase.hash(&mut s);
        let token = format!("{:016x}", s.finish());

        FClient {
            name,
            id,
            token,
            send_queue: VecDeque::new(),
            receive_queue: VecDeque::new(),
            last_receive_sent_request: Utc::now(),
        }
    }

    fn send_message(&mut self, sender_id: u64, receiver: ChatReceiver, content: String) {
        self.send_queue.push_back(ChatMessage {
            sender_id,
            receiver,
            content,
            store_date: Utc::now(),
        });
    }

    fn get_received_messages(&self) -> Vec<ChatMessage> {
        self.receive_queue
            .iter()
            .filter(|m| m.store_date >= self.last_receive_sent_request)
            .map(|m| m.clone())
            .collect()
    }

    fn register_message_receiving(&mut self) {
        self.last_receive_sent_request = Utc::now();
    }
}

struct FServer {
    clients: Vec<FClient>,
    name: String,
}

impl FServer {
    fn new(config: &ServerConfiguration) -> FServer {
        FServer {
            clients: vec![],
            name: config.name.clone(),
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

    /// Gets a Some(Client) if a client exists with the
    /// specified ID, or return None
    fn get_client(&self, client_id: u64) -> Option<&FClient> {
        self.clients.iter().find(|c| c.id == client_id)
    }

    fn get_client_mut(&mut self, client_id: u64) -> Option<&mut FClient> {
        self.clients.iter_mut().find(|c| c.id == client_id)
    }

    fn get_client_id_from_token(&self, token: &str) -> Option<u64> {
        self.clients.iter().find(|c| c.token == token).map(|c| c.id)
    }

    fn send_message_to_all(&mut self, sender_id: u64, content: String) {
        self.clients.iter_mut().for_each(|c| {
            c.send_message(sender_id, ChatReceiver::All, content.clone());
        })
    }
}

////////

///////////

/// Errors that can happen if you try to login and
/// generate the token
pub enum LoginError {}

/// General Errors
pub enum QueryError {
    InvalidToken,
    ClientNotFound,
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
    LogOff(LoginInfo),

    GetServerInfo,
    GetClientCount,
    GetClientInfo(u64),

    SendMessage(String, ChatReceiver, String),
    ReceiveMessages(String),
}

pub enum FResponseMessage {
    Login(Result<LoginResult, LoginError>),
    ValidateLogin(Result<LoginResult, QueryError>),
    LogOff(Option<u64>),

    GetServerInfo(ServerInfo),
    GetClientInfo(Option<ClientInfo>),
    GetClientCount(usize),

    SendMessage,
    ReceiveMessages(Vec<ChatMessage>),
}

pub type FMessage = (FRequestMessage, oneshot::Sender<FResponseMessage>);

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
                            QueryError::InvalidToken,
                        )));
                    }
                },
                FRequestMessage::GetServerInfo => {
                    response.send(FResponseMessage::GetServerInfo(ServerInfo::from(&server)));
                }
                FRequestMessage::GetClientInfo(cid) => {
                    response.send(FResponseMessage::GetClientInfo(
                        server
                            .get_client(cid)
                            .and_then(|c| Some(ClientInfo::from(c))),
                    ));
                }
                FRequestMessage::GetClientCount => {
                    response.send(FResponseMessage::GetClientCount(server.clients.len()));
                }
                FRequestMessage::SendMessage(sender_token, receiver, content) => {
                    let sender = server.get_client_id_from_token(&sender_token).unwrap();

                    match receiver {
                        ChatReceiver::All => {}
                        ChatReceiver::Team => {
                            unimplemented!();
                        }
                        ChatReceiver::Client(id) => {
                            server.get_client_mut(id).and_then(|c| {
                                c.send_message(sender, ChatReceiver::Client(id), content);
                                Some(c)
                            });
                        }
                    }

                    response.send(FResponseMessage::SendMessage);
                }
                FRequestMessage::ReceiveMessages(sender_token) => {
                    let client_id = server.get_client_id_from_token(&sender_token).unwrap();
                    match server.get_client_mut(client_id) {
                        Some(client) => {
                            let msgs = client.get_received_messages();
                            client.register_message_receiving();
                            response.send(FResponseMessage::ReceiveMessages(msgs));
                        }
                        None => panic!("no client"),
                    }
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
) -> Result<LoginResult, QueryError> {
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

pub async fn send_client_count_message(sender: &mut Sender<FMessage>) -> Result<usize, QueryError> {
    let (resp_tx, resp_rx) = oneshot::channel();

    sender
        .send((FRequestMessage::GetClientCount, resp_tx))
        .await
        .ok()
        .unwrap();
    let res = resp_rx.await.unwrap();

    if let FResponseMessage::GetClientCount(res) = res {
        Ok(res)
    } else {
        panic!("Unexpected response while trying to login");
    }
}

pub async fn send_info_message(
    sender: &mut Sender<FMessage>,
    token: &str,
) -> Result<ServerInfo, QueryError> {
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

pub async fn send_get_client_message(
    sender: &mut Sender<FMessage>,
    token: &str,
    client_id: u64,
) -> Result<ClientInfo, QueryError> {
    match send_validate_login_message(sender, &token).await {
        Ok(login) => {
            let (resp_tx, resp_rx) = oneshot::channel();

            sender
                .send((FRequestMessage::GetClientInfo(client_id), resp_tx))
                .await
                .ok()
                .unwrap();
            let res = resp_rx.await.unwrap();

            if let FResponseMessage::GetClientInfo(res) = res {
                match res {
                    Some(c) => Ok(c),
                    None => Err(QueryError::ClientNotFound),
                }
            } else {
                panic!("Unexpected response while trying to login");
            }
        }
        Err(e) => Err(e),
    }
}
