use crate::chat::ChatMessage;
use crate::server::ServerInfo;
use std::collections::HashMap;
use std::collections::VecDeque;

pub struct ServerState {
    pub info: ServerInfo,
    pub chats: HashMap<usize /* id */, VecDeque<ChatMessage>>,
}
