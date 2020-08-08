use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Deserialize, Serialize, PartialEq)]
pub enum ClientType {
    #[serde(rename = "normal")]
    Normal,

    #[serde(rename = "moderator")]
    Moderator,
}

// A lightweigt client class, with only the parts that we can
// send to other clients.
#[derive(Serialize, Deserialize)]
pub struct ClientInfo {
    pub id: usize,
    pub name: String,
    pub ctype: ClientType,
}

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
    pub clients: Vec<ClientInfo>,
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

    pub fn update_clients(&mut self, v: Vec<ClientInfo>) {
        self.clients = v
    }

    pub fn is_client_list_full(&self) -> bool {
        self.clients.len() >= self.max_clients
    }

    pub fn check_password(&self, pwd: &str) -> bool {
        (pwd == &self.host_password)
    }
}
