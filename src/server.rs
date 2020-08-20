use crate::client::{Client, ClientType};
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Deserialize, Serialize)]
pub enum GameStatus {
    #[serde(rename = "in_lobby")]
    InLobby,

    #[serde(rename = "starting")]
    Starting,

    #[serde(rename = "in_progress")]
    InProgress,

    #[serde(rename = "ended")]
    Ended,
}

/// TODO: list only the map hashes on the /info endpointx

#[derive(Serialize, Deserialize)]
pub struct Map {
    name: String,
}

#[derive(Serialize, Deserialize)]
pub struct ServerInfo {
    name: String,
    version: String,
    description: String,
    max_clients: usize,
    clients: Vec<Client>,
    status: GameStatus,

    map: Map,

    #[serde(skip_serializing, skip_deserializing)]
    host_password: String,
}

/// A structure that serves only to be deserialized and shown in the
/// discovery message response.
/// It contains many fields that we already have in ServerInfo, but
/// hides some others, and for that reason alone I created this struct
#[derive(Serialize, Deserialize)]
pub struct ServerDiscoveryInfo {
    name: String,
    version: String,
    description: String,
    client_count: usize,
    max_clients: usize,
    status: GameStatus,
}

impl From<&ServerInfo> for ServerDiscoveryInfo {
    fn from(s: &ServerInfo) -> Self {
        ServerDiscoveryInfo {
            name: s.name.clone(),
            version: s.version.clone(),
            description: s.description.clone(),
            client_count: s.clients.len(),
            max_clients: s.max_clients,
            status: s.status,
        }
    }
}

impl ServerInfo {
    pub fn new(
        name: &str,
        description: &str,
        max_clients: usize,
        host_password: &str,
    ) -> ServerInfo {
        ServerInfo {
            name: String::from(name),
            version: String::from("0.1.99"),
            description: String::from(description),
            max_clients,
            clients: vec![],
            status: GameStatus::InLobby,
            host_password: String::from(host_password),
            map: Map {
                name: String::from("map08"),
            },
        }
    }

    pub fn add_client(&mut self, c: Client) {
        self.clients.push(c);
    }

    pub fn remove_client(&mut self, id: usize) {
        self.clients.retain(|c| c.id() != id);
    }

    pub fn update_client(&mut self, idx: usize, nc: Client) {
        self.clients[idx] = nc;
    }

    pub fn get_clients(&self) -> &Vec<Client> {
        &self.clients
    }

    pub fn is_client_list_full(&self) -> bool {
        self.clients.len() >= self.max_clients
    }

    pub fn check_password(&self, pwd: &str) -> bool {
        (pwd == &self.host_password)
    }
}
