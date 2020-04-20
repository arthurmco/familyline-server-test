use serde::{Deserialize, Serialize, Serializer};

#[derive(Serialize, Deserialize)]
pub struct Client {
    name: String,
}

#[derive(Deserialize)]
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
            InLobby => serializer.serialize_str("in_lobby"),
            Starting => serializer.serialize_str("starting"),
            InProgress => serializer.serialize_str("in_progress"),
            Ended => serializer.serialize_str("ended")
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
    description: String,
    max_clients: usize,
    clients: Vec<Client>,
    status: GameStatus,
    map: Map,
}

impl ServerInfo {
    pub fn new(name: &str, description: &str, max_clients: usize) -> ServerInfo {
        ServerInfo {
            name: String::from(name),
            description: String::from(description),
            max_clients,
            clients: vec![],
            status: GameStatus::InLobby,
            map: Map {
                name: String::from("map08"),
            },
        }
    }
}
