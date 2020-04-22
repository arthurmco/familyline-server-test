use serde::{Deserialize, Serialize, Serializer};

// A lightweigt client class, with only the parts that we can
// send to other clients.
#[derive(Serialize, Deserialize)]
pub struct ClientInfo {
    pub id: usize,
    pub name: String,
}

#[derive(Copy, Clone, Deserialize)]
pub enum GameStatus {
    InLobby,
    Starting,
    InProgress,
    Ended,
}

impl Serialize for GameStatus {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            GameStatus::InLobby => serializer.serialize_str("in_lobby"),
            GameStatus::Starting => serializer.serialize_str("starting"),
            GameStatus::InProgress => serializer.serialize_str("in_progress"),
            GameStatus::Ended => serializer.serialize_str("ended"),
        }
    }
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
    clients: Vec<ClientInfo>,
    status: GameStatus,

    map: Map,
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
    pub fn new(name: &str, description: &str, max_clients: usize) -> ServerInfo {
        ServerInfo {
            name: String::from(name),
            version: String::from("0.1.99"),
            description: String::from(description),
            max_clients,
            clients: vec![],
            status: GameStatus::InLobby,
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
}
