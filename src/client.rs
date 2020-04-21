use rand::Rng;

use crate::server::{ClientInfo, ServerInfo};
use serde::{Deserialize, Serialize, Serializer};
use serde_json::Value;
use std::error::Error;
use std::fmt::{Display, Formatter};

/// The current state of the client
///
/// The client is the player, basically
#[derive(Debug)]
pub struct Client {
    id: usize,
    code_seed: usize,

    name: String,

    /// If true, the client is also connected to the native game
    /// protocol. Usually means that the match already begun
    connected_native: bool,
}

/// A client error
#[derive(Debug, Clone)]
pub enum ClientError {
    /// The client has no authorization to perform this action
    /// for example, downloading a map, or entering a password
    /// protected server
    Unauthorized,

    /// The resource you are asking does not exist
    ResourceNotExist,

    /// Something happened with the server
    ServerFailure,

    /// The endpoint is unknown
    UnknownEndpoint,
}

// Generation of an error is completely separate from how it is displayed.
// There's no need to be concerned about cluttering complex logic with the display style.
//
// Note that we don't store any extra info about the errors. This means we can't state
// which string failed to parse without modifying our types to carry that information.
impl Display for ClientError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "client error: {}", self)
    }
}

// This is important for other errors to wrap this one.
impl Error for ClientError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        // Generic error, underlying cause isn't tracked.
        None
    }
}

pub struct ClientResponse {
    pub headers: Vec<String>,
    pub body: String,
}

impl Client {
    /// The client code, for identifying the current client
    pub fn code(&self) -> String {
        format!(
            "{}-{}-{}",
            self.name,
            self.name.len() as u64,
            self.code_seed
        )
    }

    pub fn id(&self) -> usize {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    /// Call when the client first connect, in the /login endpoint
    ///
    ///
    pub fn new(name: &str) -> Client {
        let mut rng = rand::thread_rng();

        Client {
            id: rng.gen(),
            code_seed: rng.gen(),
            name: String::from(name),
            connected_native: false,
        }
    }

    fn handle_info_endpoint(&self, sinfo: &ServerInfo) -> Result<ClientResponse, ClientError> {
        match serde_json::to_string(&sinfo) {
            Ok(res) => Ok(ClientResponse {
                headers: vec![format!("Content-Length: {}", res.len())],
                body: res,
            }),
            Err(_) => Err(ClientError::ServerFailure),
        }
    }

    fn handle_connect_endpoint(&self) -> Result<ClientResponse, ClientError> {
        Ok(ClientResponse {
            headers: vec![
                "Connection: upgrade".to_string(),
                "Upgrade: familyline-server-protocol/0.0.1".to_string(),
            ],
            body: "!FL CONNECT localhost 32000\n\n".to_string(),
        })
    }

    fn handle_map_endpoint(&self, map: &str) -> Result<ClientResponse, ClientError> {
        if map == "test01.fmap" {
            return Ok(ClientResponse {
                headers: vec![
                    "Content-Length: 30".to_string(),
                    "Content-Type: application/octet-stream".to_string(),
                ],
                body: "FMAPAAAAAAAAAAAAAAAAAAAAAAAAAAA".to_string(),
            });
        } else {
            return Err(ClientError::Unauthorized);
        }
    }

    fn handle_map_list_endpoint(&self) -> Result<ClientResponse, ClientError> {
        #[derive(Serialize, Deserialize)]
        struct MapFile {
            name: String,
            hash: String,
            filename: String,
        }

        let map_list = vec![
            MapFile {
                name: "test01".to_string(),
                hash: "1234567890".to_string(),
                filename: "test01.fmap".to_string(),
            },
            MapFile {
                name: "test02".to_string(),
                hash: "2234567891".to_string(),
                filename: "test02.fmap".to_string(),
            },
        ];

        match serde_json::to_string(&map_list) {
            Ok(res) => Ok(ClientResponse {
                headers: vec![format!("Content-Length: {}", res.len())],
                body: res,
            }),
            Err(_) => Err(ClientError::ServerFailure),
        }
    }

    /// Handle the url as called by this client
    pub fn handle_url(&self, url: &str, sinfo: &ServerInfo) -> Result<ClientResponse, ClientError> {
        if url == "/info" {
            return self.handle_info_endpoint(&sinfo);
        }

        if url == "/connect" {
            return self.handle_connect_endpoint();
        }

        if url.starts_with("/maps") {
            return match url.splitn(3, '/').nth(2) {
                Some(map) => self.handle_map_endpoint(&map),
                None => self.handle_map_list_endpoint()
            };
        }

        return Err(ClientError::UnknownEndpoint);
    }
}

impl From<&Client> for ClientInfo {
    fn from(c: &Client) -> Self {
        ClientInfo {
            id: c.id(),
            name: c.name().to_string(),
        }
    }
}
