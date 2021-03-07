use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use std::collections::VecDeque;
use rand::Rng;

use chrono::{offset::Utc, DateTime};

use crate::config::ServerConfiguration;

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

/// Client state
#[derive(PartialEq, Copy, Clone)]
pub enum ClientState {
    /// Client just connected, it is in the game setup screen
    InGameSetup,

    /// Client is ready to start player
    InReady,

    /// All clients are ready, game is starting
    InGameStart,

    /// Client connected to the game socket; the game is started or already started
    InGame,
}

pub struct FClient {
    pub name: String,
    pub id: u64,
    pub token: String,
    state: ClientState,

    last_receive_sent_request: DateTime<Utc>,
    send_queue: VecDeque<ChatMessage>,
    receive_queue: VecDeque<ChatMessage>,
}

impl FClient {
    pub fn new(name: String) -> FClient {
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
            state: ClientState::InGameSetup,
            send_queue: VecDeque::new(),
            receive_queue: VecDeque::new(),
            last_receive_sent_request: Utc::now(),
        }
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn send_message(&mut self, sender_id: u64, receiver: ChatReceiver, content: String) {
        self.send_queue.push_back(ChatMessage {
            sender_id,
            receiver,
            content,
            store_date: Utc::now(),
        });
    }

    pub fn get_state(&self) -> ClientState {
        self.state
    }

    /// Notify that the client is ready
    ///
    /// Return true if the state change is valid, false if it is not.
    pub fn set_ready(&mut self) -> bool {
        let (nstate, ret) = match self.state {
            ClientState::InReady => (ClientState::InReady, true),
            ClientState::InGameSetup => (ClientState::InReady, true),
            _ => (self.state, false),
        };

        self.state = nstate;
        return ret;
    }

    /// Notify that the client is not ready anymore
    ///
    /// Return true if the state change is valid, false if it is not.
    pub fn unset_ready(&mut self) -> bool {
        let (nstate, ret) = match self.state {
            ClientState::InReady => (ClientState::InGameSetup, true),
            ClientState::InGameSetup => (ClientState::InGameSetup, true),
            _ => (self.state, false),
        };

        self.state = nstate;
        return ret;
    }

    pub fn get_received_messages(&self) -> Vec<ChatMessage> {
        self.receive_queue
            .iter()
            .filter(|m| m.store_date >= self.last_receive_sent_request)
            .map(|m| m.clone())
            .collect()
    }

    pub fn register_message_receiving(&mut self) {
        self.last_receive_sent_request = Utc::now();
    }
}

pub struct FServer {
    pub clients: Vec<FClient>,
    pub name: String,
}

impl FServer {
    pub fn new(config: &ServerConfiguration) -> FServer {
        FServer {
            clients: vec![],
            name: config.name.clone(),
        }
    }

    /// Add a client to the server database, returns a reference to it
    pub fn add_client(&mut self, c: FClient) -> &FClient {
        self.clients.push(c);
        self.clients.last().unwrap()
    }

    /// Remove a client from the server database
    ///
    /// Returns its ID if it existed, None if it did not.
    pub fn remove_client(&mut self, id: u64) -> Option<u64> {
        let idx = self.clients.iter().enumerate().find(|&(idx, v)| v.id == id);

        match idx {
            Some((vidx, _)) => {
                let v = self.clients.remove(vidx);
                Some(v.id)
            }
            None => None,
        }
    }

    /// Set a client ready.
    ///
    /// Return true if the set was successful, false if it was not
    pub fn set_client_ready(&mut self, client_id: u64) -> bool {
        match self.get_client_mut(client_id) {
            Some(c) => c.set_ready(),
            None => false,
        }
    }

    /// Unet a client ready.
    ///
    /// Return true if the set was successful, false if it was not
    pub fn unset_client_ready(&mut self, client_id: u64) -> bool {
        match self.get_client_mut(client_id) {
            Some(c) => c.unset_ready(),
            None => false,
        }
    }

    /// Is all clients ready to connect?
    /// 
    /// This means more than one client, and all clients ready
    pub fn is_ready_to_connect(&self) -> bool {
        if self.clients.len() < 2 {
            false
        } else {
            self.clients.iter().all(|c| c.get_state() == ClientState::InReady)
        }
    }

    /// Check what user do this token belongs
    ///
    /// If it belongs to some user, return it
    /// Else, returns None
    pub fn validate_token(&self, token: &str) -> Option<&FClient> {
        self.clients.iter().find(|c| c.token == token)
    }

    /// Gets a Some(Client) if a client exists with the
    /// specified ID, or return None
    pub fn get_client(&self, client_id: u64) -> Option<&FClient> {
        self.clients.iter().find(|c| c.id == client_id)
    }

    pub fn get_client_mut(&mut self, client_id: u64) -> Option<&mut FClient> {
        self.clients.iter_mut().find(|c| c.id == client_id)
    }

    pub fn get_client_id_from_token(&self, token: &str) -> Option<u64> {
        self.clients.iter().find(|c| c.token == token).map(|c| c.id)
    }

    pub fn send_message_to_all(&mut self, sender_id: u64, content: String) {
        self.clients.iter_mut().for_each(|c| {
            c.send_message(sender_id, ChatReceiver::All, content.clone());
        })
    }
}
