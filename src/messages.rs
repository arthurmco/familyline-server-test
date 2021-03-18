use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::sync::oneshot;

use crate::client::*;
use crate::config::ServerConfiguration;
use crate::gamemsg::{Packet, PacketMessage};
use chrono::prelude::*;
use serde::{Deserialize, Serialize};

/// Errors that can happen if you try to login and
/// generate the token
pub enum LoginError {}

/// General Errors
#[derive(Debug)]
pub enum QueryError {
    InvalidToken,
    ClientNotFound,

    /// You tried to call /connect, but not all clients were ready
    NotAllClientsReady,
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
    ready: bool,
    connecting: bool,
    ingame: bool,
}

impl From<&FClient> for ClientInfo {
    fn from(c: &FClient) -> Self {
        ClientInfo {
            name: c.name.clone(),
            user_id: c.id,
            ready: c.get_state() == ClientState::InReady,
            connecting: c.get_state() == ClientState::InGameStart,
            ingame: c.get_state() == ClientState::InGame,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ServerInfo {
    name: String,
    clients: Vec<ClientInfo>,
    max_clients: usize,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ConnectInfo {
    pub address: String,
    pub port: u16,
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
    LogOff(String),

    GetServerInfo,
    GetClientCount,
    GetClientInfo(u64),

    SetReady(u64),
    UnsetReady(u64),

    ConnectStart(String),

    SendMessage(String, ChatReceiver, String),
    ReceiveMessages(String),

    ConnectConfirm(String),
    CheckAllClientsConnected(String),
    PushGameMessage(String, Packet),
    PopGameMessage(String),
}

pub enum FResponseMessage {
    Login(Result<LoginResult, LoginError>),
    ValidateLogin(Result<LoginResult, QueryError>),
    LogOff(Option<u64>),

    GetServerInfo(ServerInfo),
    GetClientInfo(Option<ClientInfo>),
    GetClientCount(usize),

    SetReady(bool),
    UnsetReady(bool),

    ConnectStart(Result<ConnectInfo, QueryError>),

    SendMessage,
    ReceiveMessages(Vec<ChatMessage>),

    ConnectConfirm(Result<(), QueryError>),
    CheckAllClientsConnected(bool),
    PushGameMessage(bool),
    PopGameMessage(Option<Packet>),
}

pub type FMessage = (FRequestMessage, oneshot::Sender<FResponseMessage>);

/// Open a request channel
///
/// Both the HTTP login API and the game server will use it
/// to request data.
///
/// This function will receive all messages (the questions
/// are defined on FRequestMessage) and answer them (the responses
/// are defined on FResponseMessage). Both are a few lines above
///
/// It will become very big
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
                FRequestMessage::LogOff(token) => match server.validate_token(&token) {
                    Some(client) => {
                        let cid = client.id();
                        match server.remove_client(cid) {
                            Some(id) => response.send(FResponseMessage::LogOff(Some(cid))),
                            None => response.send(FResponseMessage::LogOff(None)),
                        };
                    }
                    None => {
                        response.send(FResponseMessage::LogOff(None));
                    }
                },
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
                FRequestMessage::SetReady(cid) => {
                    response.send(FResponseMessage::SetReady(server.set_client_ready(cid)));
                }
                FRequestMessage::UnsetReady(cid) => {
                    response.send(FResponseMessage::UnsetReady(server.unset_client_ready(cid)));
                }
                FRequestMessage::ConnectStart(token) => match server.validate_token(&token) {
                    Some(_) => {
                        if server.is_ready_to_connect() {
                            response.send(FResponseMessage::ConnectStart(Ok(ConnectInfo {
                                address: String::default(),
                                port: 0,
                            })));
                        } else {
                            response.send(FResponseMessage::ConnectStart(Err(
                                QueryError::NotAllClientsReady,
                            )));
                        }
                    }
                    None => {
                        response.send(FResponseMessage::ConnectStart(Err(
                            QueryError::InvalidToken,
                        )));
                    }
                },
                // Set the connection state and tell all other clients
                // that you connected
                FRequestMessage::ConnectConfirm(token) => match server.validate_token(&token) {
                    Some(client) => {
                        let id = client.id();
                        server.set_client_connect(id);

                        let timestamp = Utc::now().timestamp();
                        server.broadcast_game_packet(
                            Packet::new(
                                0,
                                0,
                                id,
                                timestamp as u64,
                                1,
                                PacketMessage::StartResponse(id, server.all_clients_connected()),
                            ),
                            vec![],
                        );
                        response.send(FResponseMessage::ConnectConfirm(Ok(())));
                    }
                    None => {
                        response.send(FResponseMessage::ConnectConfirm(Err(
                            QueryError::InvalidToken,
                        )));
                    }
                },
                FRequestMessage::CheckAllClientsConnected(token) => {
                    match server.validate_token(&token) {
                        Some(_) => {
                            response.send(FResponseMessage::CheckAllClientsConnected(
                                server.all_clients_connected(),
                            ));
                        }
                        None => {
                            response.send(FResponseMessage::CheckAllClientsConnected(false));
                        }
                    }
                }
                FRequestMessage::PushGameMessage(token, packet) => {
                    let cid = match server.validate_token(&token) {
                        Some(client) => client.id(),
                        None => 0,
                    };

                    if cid > 0 {
                        let rres = match packet.message {
                            PacketMessage::LoadingRequest(_) => {
                                if !server.set_client_starting(cid) {
                                    eprintln!("Invalid state transition to starting ({:?})", cid);
                                    false
                                } else {
                                    true
                                }
                            }
                            PacketMessage::GameStartRequest => {
                                if !server.set_client_start_to_receive_inputs(cid) {
                                    eprintln!("Invalid state transition to game ({:?})", cid);
                                    false
                                } else {
                                    true
                                }
                            }
                            PacketMessage::SendInputRequest(_, _) => {
                                match server.get_client(cid).unwrap().get_state() {
                                    ClientState::InGame => true,
                                    _ => {
                                        eprintln!("Invalid state for receiving this message");
                                        false
                                    }
                                }
                            }
                            _ => true,
                        };

                        if rres {
                            server.act_on_packet(&packet);
                        }

                        if packet.source_client == packet.dest_client {
                            // cannot send a message to yourself.
                            response.send(FResponseMessage::PushGameMessage(false));
                        } else if packet.dest_client == 0 {
                            // handle the broadcasting case
                            if rres {
                                server.broadcast_game_packet(packet, vec![cid]);
                            }
                            response.send(FResponseMessage::PushGameMessage(rres));
                        } else {
                            if rres {
                                server.push_game_packet(packet.dest_client, packet);
                            }
                            response.send(FResponseMessage::PushGameMessage(rres));
                        }
                    } else {
                        response.send(FResponseMessage::PushGameMessage(false));
                    }
                }
                FRequestMessage::PopGameMessage(token) => match server.validate_token(&token) {
                    Some(client) => {
                        let id = client.id();
                        response.send(FResponseMessage::PopGameMessage(server.pop_game_packet(id)));
                    }
                    None => {
                        response.send(FResponseMessage::PopGameMessage(None));
                    }
                },
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
        Ok(_) => {
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

pub async fn send_logout_message(
    sender: &mut Sender<FMessage>,
    token: &str,
) -> Result<u64, QueryError> {
    match send_validate_login_message(sender, &token).await {
        Ok(_) => {
            let (resp_tx, resp_rx) = oneshot::channel();

            sender
                .send((FRequestMessage::LogOff(String::from(token)), resp_tx))
                .await
                .ok()
                .unwrap();
            let res = resp_rx.await.unwrap();

            if let FResponseMessage::LogOff(res) = res {
                match res {
                    Some(val) => Ok(val),
                    None => panic!("Invalid token!"),
                }
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
        Ok(_) => {
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

pub async fn send_set_ready_message(
    sender: &mut Sender<FMessage>,
    token: &str,
) -> Result<bool, QueryError> {
    match send_validate_login_message(sender, &token).await {
        Ok(login) => {
            let (resp_tx, resp_rx) = oneshot::channel();

            sender
                .send((FRequestMessage::SetReady(login.user_id), resp_tx))
                .await
                .ok()
                .unwrap();
            let res = resp_rx.await.unwrap();

            if let FResponseMessage::SetReady(res) = res {
                Ok(res)
            } else {
                panic!("Unexpected response while trying to login");
            }
        }
        Err(e) => Err(e),
    }
}

pub async fn send_unset_ready_message(
    sender: &mut Sender<FMessage>,
    token: &str,
) -> Result<bool, QueryError> {
    match send_validate_login_message(sender, &token).await {
        Ok(login) => {
            let (resp_tx, resp_rx) = oneshot::channel();

            sender
                .send((FRequestMessage::UnsetReady(login.user_id), resp_tx))
                .await
                .ok()
                .unwrap();
            let res = resp_rx.await.unwrap();

            if let FResponseMessage::UnsetReady(res) = res {
                Ok(res)
            } else {
                panic!("Unexpected response while trying to login");
            }
        }
        Err(e) => Err(e),
    }
}

pub async fn send_connect_message(
    sender: &mut Sender<FMessage>,
    token: &str,
) -> Result<ConnectInfo, QueryError> {
    match send_validate_login_message(sender, &token).await {
        Ok(_) => {
            let (resp_tx, resp_rx) = oneshot::channel();

            sender
                .send((FRequestMessage::ConnectStart(String::from(token)), resp_tx))
                .await
                .ok()
                .unwrap();
            let res = resp_rx.await.unwrap();

            if let FResponseMessage::ConnectStart(res) = res {
                res
            } else {
                panic!("Unexpected response while trying to login");
            }
        }
        Err(e) => Err(e),
    }
}

pub async fn send_connect_confirm_message(
    sender: &mut Sender<FMessage>,
    token: &str,
) -> Result<(), QueryError> {
    match send_validate_login_message(sender, &token).await {
        Ok(_) => {
            let (resp_tx, resp_rx) = oneshot::channel();

            sender
                .send((
                    FRequestMessage::ConnectConfirm(String::from(token)),
                    resp_tx,
                ))
                .await
                .ok()
                .unwrap();
            let res = resp_rx.await.unwrap();

            if let FResponseMessage::ConnectConfirm(res) = res {
                res
            } else {
                panic!("Unexpected response on ConnectConfirm")
            }
        }
        Err(e) => Err(e),
    }
}

///
/// Pop the game packet for the client whose token is `token`
///
/// Return the packet data, or an error
/// We can have no packet, in this case it will return a None.
/// Otherwise, a Some(...) will be returned.
pub async fn send_pop_game_packet_message(
    sender: &mut Sender<FMessage>,
    token: &str,
) -> Result<Option<Packet>, QueryError> {
    match send_validate_login_message(sender, &token).await {
        Ok(_) => {
            let (resp_tx, resp_rx) = oneshot::channel();

            sender
                .send((
                    FRequestMessage::PopGameMessage(String::from(token)),
                    resp_tx,
                ))
                .await
                .ok()
                .unwrap();
            let res = resp_rx.await.unwrap();

            if let FResponseMessage::PopGameMessage(res) = res {
                Ok(res)
            } else {
                panic!("Unexpected response on send_pop_game_packet_message")
            }
        }
        Err(e) => Err(e),
    }
}

pub async fn send_push_game_packet_message(
    sender: &mut Sender<FMessage>,
    token: &str,
    p: Packet,
) -> Result<bool, QueryError> {
    match send_validate_login_message(sender, &token).await {
        Ok(_) => {
            let (resp_tx, resp_rx) = oneshot::channel();

            sender
                .send((
                    FRequestMessage::PushGameMessage(String::from(token), p),
                    resp_tx,
                ))
                .await
                .ok()
                .unwrap();
            let res = resp_rx.await.unwrap();

            if let FResponseMessage::PushGameMessage(res) = res {
                Ok(res)
            } else {
                panic!("Unexpected response on send_push_game_packet_message")
            }
        }
        Err(e) => Err(e),
    }
}

pub async fn send_check_all_clients_connected_message(
    sender: &mut Sender<FMessage>,
    token: &str,
) -> Result<bool, QueryError> {
    match send_validate_login_message(sender, &token).await {
        Ok(_) => {
            let (resp_tx, resp_rx) = oneshot::channel();

            sender
                .send((
                    FRequestMessage::CheckAllClientsConnected(String::from(token)),
                    resp_tx,
                ))
                .await
                .ok()
                .unwrap();
            let res = resp_rx.await.unwrap();

            if let FResponseMessage::CheckAllClientsConnected(res) = res {
                Ok(res)
            } else {
                panic!("Unexpected response on send_check_all_clients_connected_message")
            }
        }
        Err(e) => Err(e),
    }
}
