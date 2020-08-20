use rand::Rng;

use crate::server::ServerInfo;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug, Copy, Clone, Deserialize, Serialize, PartialEq)]
pub enum ClientType {
    #[serde(rename = "normal")]
    Normal,

    #[serde(rename = "moderator")]
    Moderator,
}

/// The current state of the client
///
/// The client is the player, basically
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Client {
    id: usize,

    #[serde(skip)]
    code_seed: usize,

    name: String,

    /// If true, the client is also connected to the native game
    /// protocol. Usually means that the match already begun
    #[serde(skip)]
    connected_native: bool,

    ctype: ClientType,
}

/// A client error
#[derive(Debug, Clone)]
pub enum ClientError {
    /// The input the user gave was incorrect
    BadInput,

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

    pub fn get_client_type(&self) -> ClientType {
        self.ctype
    }

    pub fn set_client_type(&mut self, c: ClientType) {
        self.ctype = c;
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
            ctype: ClientType::Normal,
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

    fn handle_ready_endpoint(&self) -> Result<ClientResponse, ClientError> {
        Ok(ClientResponse {
            headers: vec![
                "Connection: upgrade".to_string(),
                "Upgrade: familyline-server-protocol/0.0.1".to_string(),
            ],
            body: "!FL READY localhost 32000\n\n".to_string(),
        })
    }

    fn handle_host_endpoint(
        &mut self,
        sinfo: &ServerInfo,
        body: &str,
    ) -> Result<ClientResponse, ClientError> {
        let sbody = body.chars().filter(|&c| c != '\0').collect::<String>();

        #[derive(Serialize, Deserialize)]
        struct ClientHostRequest {
            password: String,
        };

        // Do not allow setting the host again in this endpoint
        if sinfo
            .get_clients()
            .iter()
            .filter(|c| c.ctype == ClientType::Moderator)
            .count()
            > 0
        {
            return Err(ClientError::BadInput);
        }

        let ahost_req: ClientHostRequest = serde_json::from_str(&sbody).unwrap();
        let host_req: ClientHostRequest = match serde_json::from_str(&sbody) {
            Ok(s) => s,
            Err(_) => return Err(ClientError::BadInput),
        };

        if sinfo.check_password(&host_req.password) {
            self.ctype = ClientType::Moderator;
            return Ok(ClientResponse {
                headers: vec![],
                body: String::from(""),
            });
        } else {
            return Err(ClientError::Unauthorized);
        }
    }

    fn handle_map_endpoint(&self, map: &str) -> Result<ClientResponse, ClientError> {
        if map == "test01.fmap" {
            return Ok(ClientResponse {
                headers: vec!["Content-Type: application/octet-stream".to_string()],
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

    /// Check if a endpoint has a mutable handler
    pub fn is_url_mutable(&self, url: &str) -> bool {
        // Please change this when the list gets big.

        if url.starts_with("/setmod") {
            return true;
        }

        return false;
    }

    /// Handle the url as called by this client
    pub fn handle_url(
        &self,
        url: &str,
        sinfo: &ServerInfo,
        body: &str,
    ) -> Result<ClientResponse, ClientError> {
        if url == "/info" {
            return self.handle_info_endpoint(&sinfo);
        }

        if url == "/ready" {
            return self.handle_ready_endpoint();
        }

        if url.starts_with("/maps") {
            return match url.splitn(3, '/').nth(2) {
                Some(map) => self.handle_map_endpoint(&map),
                None => self.handle_map_list_endpoint(),
            };
        }

        return Err(ClientError::UnknownEndpoint);
    }

    pub fn handle_url_mut(
        &mut self,
        url: &str,
        sinfo: &ServerInfo,
        body: &str,
    ) -> Result<ClientResponse, ClientError> {
        if url == "/setmod" {
            return self.handle_host_endpoint(&sinfo, body);
        }

        return Err(ClientError::UnknownEndpoint);
    }
}
