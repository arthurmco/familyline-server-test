use rand::Rng;

use crate::server::ServerInfo;
use serde_json::Value;
use std::error::Error;
use std::fmt::{Display, Formatter};

/// The current state of the client
///
/// The client is the player, basically
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

struct ClientResponse {
    headers: Vec<String>,
    body: String,
}

impl Client {
    /// The client code, for identifying the current client
    pub fn code(&self) -> String {
        format!(
            "{}-{}-{}",
            self.name,
            self.id * self.name.len(),
            self.code_seed
        )
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
                headers: vec![],
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
        if map == "08" {
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

    /// Handle the url as called by this client
    pub fn handle_url(&self, url: &str, sinfo: &ServerInfo) -> Result<ClientResponse, ClientError> {
        if url == "/info" {
            return self.handle_info_endpoint(&sinfo);
        }

        if url == "/connect" {
            return self.handle_connect_endpoint();
        }

        if url.starts_with("/map/") {
            return match url.splitn(3, '/').nth(2) {
                Some(map) => self.handle_map_endpoint(&map),
                None => Err(ClientError::UnknownEndpoint),
            };
        }

        return Err(ClientError::UnknownEndpoint);
    }
}
