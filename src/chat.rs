use crate::state::ServerState;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::VecDeque;
use std::sync::{Arc, RwLock};

/// Decode a websocket message, return the string, or None
fn parse_websocket_message(buf: &Vec<u8>) -> Option<String> {
    let is_fin = ((buf[0] >> 4) & 0x1) > 0;

    let opcode = buf[0] & 0xf;

    if opcode == 0x2 {
        // Not text
        return None;
    }

    let masked = (buf[1] >> 0x7) > 0;

    if !masked {
        // The server cannot accept unmasked client messages
        return None;
    }

    let lenfield = (buf[1] ^ 0x80);
    let mask_offset = match lenfield {
        127 => 6,
        126 => 4,
        _ => 2,
    };
    let msglen = match lenfield {
        127 => {
            let mut l = buf[5] as usize;
            l |= (buf[4] as usize) << 8;
            l |= (buf[3] as usize) << 16;
            l |= (buf[2] as usize) << 24;
            l
        }
        126 => {
            let mut l = buf[3] as usize;
            l |= (buf[2] as usize) << 8;
            l
        }
        _ => lenfield as usize,
    };

    let content_offset = mask_offset + 4;
    let mask = &buf[mask_offset..content_offset];

    let content_bytes = &buf[content_offset..];
    let content_decoded = content_bytes
        .into_iter()
        .enumerate()
        .map(|(i, x)| x ^ mask[i % 4])
        .take(msglen)
        .collect();

    match String::from_utf8(content_decoded) {
        Ok(s) => Some(s),
        Err(_) => None,
    }
}

fn build_websocket_message(data: &str) -> Vec<u8> {
    let mut v = Vec::<u8>::new();
    let bdata = data.as_bytes();

    let reallen = bdata.len();
    let fieldlen = if bdata.len() > 65536 {
        127
    } else if bdata.len() > 126 {
        126
    } else {
        reallen as u8
    };

    v.push(0x81); // FIN, opcode 0x1
    v.push(fieldlen);

    if fieldlen == 126 {
        v.push((reallen & 0xff) as u8);
        v.push((reallen >> 8) as u8);
    } else if fieldlen == 127 {
        v.push((reallen & 0xff) as u8);
        v.push(((reallen >> 8) & 0xff) as u8);
        v.push(((reallen >> 16) & 0xff) as u8);
        v.push(((reallen >> 24) & 0xff) as u8);
    }

    v.extend_from_slice(bdata);

    return v;
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ChatMessage {
    /// Tick number that the message was sent
    /// For pre-match messages, this value is 0
    pub tick: usize,

    /// The ID of the user who sent the message
    pub sender: usize,

    /// Message timestamp
    pub timestamp: usize,

    /// The chat message itself
    pub message: String,
}

fn push_message(state: &Arc<RwLock<ServerState>>, cm: &ChatMessage, client_id: usize) {
    let pushed = {
        match state.write().unwrap().chats.get_mut(&client_id) {
            Some(ref mut queue) => {
                queue.push_back(cm.clone());
                true
            }
            None => false,
        }
    };

    if !pushed {
        let mut chatqueue = VecDeque::new();
        chatqueue.push_back(cm.clone());

        state.write().unwrap().chats.insert(client_id, chatqueue);
    }
}

pub fn handle_chat_conversation(
    state: &Arc<RwLock<ServerState>>,
    buf: &Vec<u8>,
    client_id: usize,
) -> Vec<u8> {
    let s = match parse_websocket_message(buf) {
        Some(s) => s,
        None => {
            eprintln!("no valid string received");
            String::default()
        }
    };

    let chat_msg: ChatMessage = match serde_json::from_str(&s) {
        Ok(m) => m,
        Err(_) => {
            return build_websocket_message("ERROR");
        }
    };

    push_message(&state, &chat_msg, client_id);

    return send_pending_chat_messages(&state, client_id);
}

pub fn send_pending_chat_messages(state: &Arc<RwLock<ServerState>>, client_id: usize) -> Vec<u8> {
    let recvet: Vec<ChatMessage> = {
        let sread = state.read().unwrap();
        let chatqueue = sread.chats.get(&client_id);

        match chatqueue {
            Some(ref queue) => queue
                .iter()
                .filter(|c| c.sender == client_id)
                .map(|c| c.clone())
                .collect(),
            None => vec![],
        }
    };

    let messagelist = match state.write().unwrap().chats.get_mut(&client_id) {
        Some(ref mut queue) => {
            queue.retain(|c| c.sender != client_id);
            recvet
        }
        None => recvet,
    };

    if messagelist.len() == 0 {
        return vec![];
    }

    let messagestr = serde_json::to_string(&messagelist).unwrap();
    return build_websocket_message(&messagestr);
}
