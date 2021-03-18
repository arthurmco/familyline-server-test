use tokio::net::{TcpListener, TcpStream};

extern crate flatbuffers;

use crate::config::ServerConfiguration;
use crate::messages::{
    send_check_all_clients_connected_message, send_connect_confirm_message,
    send_pop_game_packet_message, send_push_game_packet_message, FMessage, FRequestMessage,
    FResponseMessage,
};
use chrono::prelude::*;
use std::time::Duration;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, Interest},
    sync::mpsc::Sender,
};

use crate::network_generated::{
    get_root_as_net_packet, GameStartRequest, GameStartRequestArgs, GameStartResponse,
    GameStartResponseArgs, InputRequest, InputRequestArgs, InputResponse, InputResponseArgs,
    LoadingRequest, LoadingRequestArgs, LoadingResponse, LoadingResponseArgs, Message, NetPacket,
    NetPacketArgs, StartRequest, StartRequestArgs, StartResponse, StartResponseArgs,
};

use crate::input_generated::InputType as SerInputType;
use crate::input_generated::{
    CameraMove, CameraMoveArgs, CameraRotate, CameraRotateArgs, CommandInput, CommandInputA,
    CommandInputAArgs, CommandInputArgs, CreateEntity, CreateEntityArgs, ObjectMove,
    ObjectMoveArgs, SelectAction, SelectActionArgs, SelectActionObjects, SelectActionObjectsArgs,
};

#[derive(Debug, Clone)]
pub enum InputType {
    /// Command input: the command name + its args
    CommandInput(String, Vec<u64>),

    /// Select action: the IDs of the selected objects
    SelectAction(Vec<u64>),

    /// Object move: where do the selected objects will move?
    ObjectMove(u64, u64),

    CameraMove(f64, f64, f64),
    CameraRotate(f64),
    CreateEntity(String, f64, f64),

    Invalid,
}

#[derive(Debug, Clone)]
pub enum PacketMessage {
    StartRequest(u64, String),
    StartResponse(u64, bool),

    LoadingRequest(u16),
    LoadingResponse(u16),

    GameStartRequest,
    GameStartResponse,

    /// Send a input
    ///
    /// The u64 is the client that originally sent the input
    /// The input type is the data of the input
    SendInputRequest(u64, InputType),

    /// Send a input, the response
    ///
    /// The u64 is the client that originally sent the input
    /// The bool is an ack field, that the server processed the
    /// packet and sent it to all clients.
    SendInputResponse(u64, bool),

    /// A request for disconnection.
    ///
    /// The u64 is the client ID of the disconnector.
    DisconnectRequest(u64),
    DisconnectResponse(u64),

    Invalid,
}

/// The packet that we will be sent to the server message queue
///
/// We cannot send the NetPacket directly, due to it being bound
/// to the same lifetime as the flatbuffers builder. The builder
/// is temporary, and, for that reason, it would not work.
#[derive(Debug, Clone)]
pub struct Packet {
    pub tick: u64,
    pub source_client: u64,
    pub dest_client: u64, // dest client, or 0 to all clients
    pub timestamp: u64,
    pub message_id: u64,

    pub message: PacketMessage,
}

impl Packet {
    pub fn new(
        tick: u64,
        source_client: u64,
        dest_client: u64,
        timestamp: u64,
        message_id: u64,
        message: PacketMessage,
    ) -> Packet {
        Packet {
            tick,
            source_client,
            dest_client,
            timestamp,
            message_id,
            message,
        }
    }

    /// Make the server own the packet
    ///
    /// Also reset the packet timestamp.
    pub fn make_server_own(&self) -> Packet {
        Packet {
            source_client: 0 as u64,
            timestamp: Utc::now().timestamp() as u64,
            message: self.message.clone(),
            ..*self
        }
    }

    /// Set the destination client of the message, and create
    /// a new packet for that
    pub fn to_new_client(&self, newclient: u64) -> Packet {
        Packet {
            dest_client: newclient,
            message: self.message.clone(),
            ..*self
        }
    }

    /// Set the packet message, and create a new packet for that.
    pub fn to_new_message(&self, msg: PacketMessage) -> Packet {
        Packet {
            message: msg,
            ..*self
        }
    }

    pub fn from_flatbuffers(p: NetPacket) -> Packet {
        Packet {
            tick: p.tick(),
            source_client: p.source_client(),
            dest_client: p.dest_client(),
            timestamp: p.timestamp(),
            message_id: p.id(),
            message: match p.message_type() {
                Message::sreq => {
                    let m = p.message_as_sreq().unwrap();
                    PacketMessage::StartRequest(
                        m.client_id(),
                        m.token().map_or(String::default(), |s| String::from(s)),
                    )
                }
                Message::sres => {
                    let m = p.message_as_sres().unwrap();
                    PacketMessage::StartResponse(m.client_ack(), m.all_clients_ack())
                }
                Message::lreq => {
                    let m = p.message_as_lreq().unwrap();
                    PacketMessage::LoadingRequest(m.percent())
                }
                Message::lres => {
                    let m = p.message_as_lres().unwrap();
                    PacketMessage::LoadingResponse(m.percent())
                }
                Message::greq => PacketMessage::GameStartRequest,
                Message::gres => PacketMessage::GameStartResponse,
                Message::ireq => {
                    let m = p.message_as_ireq().unwrap();
                    let client = m.client_from();

                    let msg = match m.input_msg_type() {
                        SerInputType::NONE => InputType::Invalid,
                        SerInputType::create => {
                            let tm = m.input_msg_as_create().unwrap();
                            InputType::CreateEntity(
                                tm.type_().map(|v| String::from(v)).unwrap_or_default(),
                                tm.x_pos() as f64,
                                tm.y_pos() as f64,
                            )
                        }
                        SerInputType::cmd => {
                            let tm = m.input_msg_as_cmd().unwrap();
                            let args: Vec<u64> = tm
                                .args()
                                .map(|a| a.args().unwrap().iter().collect())
                                .unwrap_or_default();
                            InputType::CommandInput(String::from(tm.command()), args)
                        }
                        SerInputType::cam_move => {
                            let tm = m.input_msg_as_cam_move().unwrap();
                            InputType::CameraMove(tm.x_delta(), tm.y_delta(), tm.zoom_delta())
                        }
                        SerInputType::cam_rotate => {
                            let tm = m.input_msg_as_cam_rotate().unwrap();
                            InputType::CameraRotate(tm.radians())
                        }
                        SerInputType::obj_move => {
                            let tm = m.input_msg_as_obj_move().unwrap();
                            InputType::ObjectMove(tm.x_pos() as u64, tm.y_pos() as u64)
                        }
                        SerInputType::sel => {
                            let tm = m.input_msg_as_sel().unwrap();
                            let objs: Vec<u64> = tm.objects().values().iter().collect();
                            InputType::SelectAction(objs)
                        }
                    };
                    PacketMessage::SendInputRequest(client, msg)
                }
                Message::ires => {
                    let m = p.message_as_ires().unwrap();
                    PacketMessage::SendInputResponse(m.client_from(), m.client_ack())
                }
                Message::NONE => {
                    println!("invalid message received!");
                    PacketMessage::Invalid
                }
            },
        }
    }

    fn type_to_flatbuffer(
        &self,
        builder: &mut flatbuffers::FlatBufferBuilder,
    ) -> (
        Message,
        Option<flatbuffers::WIPOffset<flatbuffers::UnionWIPOffset>>,
    ) {
        match &self.message {
            PacketMessage::StartRequest(id, token) => {
                let str = builder.create_string(&token.clone());
                let c = StartRequest::create(
                    builder,
                    &StartRequestArgs {
                        client_id: *id,
                        token: Some(str),
                    },
                );
                (Message::sreq, Some(c.as_union_value()))
            }
            PacketMessage::StartResponse(client_ack, all_clients_ack) => {
                let c = StartResponse::create(
                    builder,
                    &StartResponseArgs {
                        client_ack: *client_ack,
                        all_clients_ack: *all_clients_ack,
                    },
                );
                (Message::sres, Some(c.as_union_value()))
            }
            PacketMessage::LoadingRequest(percent) => {
                let c = LoadingRequest::create(builder, &LoadingRequestArgs { percent: *percent });
                (Message::lreq, Some(c.as_union_value()))
            }
            PacketMessage::LoadingResponse(percent) => {
                let c =
                    LoadingResponse::create(builder, &LoadingResponseArgs { percent: *percent });
                (Message::lres, Some(c.as_union_value()))
            }
            PacketMessage::GameStartRequest => {
                let c = GameStartRequest::create(builder, &GameStartRequestArgs { reserved: 0 });
                (Message::greq, Some(c.as_union_value()))
            }
            PacketMessage::GameStartResponse => {
                let c = GameStartResponse::create(builder, &GameStartResponseArgs { reserved: 0 });
                (Message::gres, Some(c.as_union_value()))
            }
            PacketMessage::SendInputRequest(client_from, mtype) => {
                let (input_msg_type, input_msg) = match mtype {
                    InputType::Invalid => (SerInputType::NONE, None),
                    InputType::CameraMove(x, y, zoom) => {
                        let mc = CameraMove::create(
                            builder,
                            &CameraMoveArgs {
                                x_delta: *x,
                                y_delta: *y,
                                zoom_delta: *zoom,
                            },
                        );
                        (SerInputType::cam_move, Some(mc.as_union_value()))
                    }
                    InputType::CameraRotate(rads) => {
                        let mc =
                            CameraRotate::create(builder, &CameraRotateArgs { radians: *rads });
                        (SerInputType::cam_rotate, Some(mc.as_union_value()))
                    }
                    InputType::CommandInput(cmd, params) => {
                        let vparams = builder.create_vector(&params[..]);
                        let scmd = builder.create_string(&cmd);
                        let cargs = CommandInputA::create(
                            builder,
                            &CommandInputAArgs {
                                args: Some(vparams),
                            },
                        );
                        let mc = CommandInput::create(
                            builder,
                            &CommandInputArgs {
                                command: Some(scmd),
                                args: Some(cargs),
                            },
                        );
                        (SerInputType::cmd, Some(mc.as_union_value()))
                    }
                    InputType::CreateEntity(etype, x, y) => {
                        let stype = builder.create_string(&etype);
                        let mc = CreateEntity::create(
                            builder,
                            &CreateEntityArgs {
                                type_: Some(stype),
                                x_pos: *x as u32,
                                y_pos: *y as u32,
                            },
                        );
                        (SerInputType::create, Some(mc.as_union_value()))
                    }
                    InputType::ObjectMove(x, y) => {
                        let mc = ObjectMove::create(
                            builder,
                            &ObjectMoveArgs {
                                x_pos: *x as u32,
                                y_pos: *y as u32,
                            },
                        );
                        (SerInputType::obj_move, Some(mc.as_union_value()))
                    }
                    InputType::SelectAction(entities) => {
                        let eparams = builder.create_vector(&entities[..]);
                        let eargs = SelectActionObjects::create(
                            builder,
                            &SelectActionObjectsArgs {
                                values: Some(eparams),
                            },
                        );
                        let mc = SelectAction::create(
                            builder,
                            &SelectActionArgs {
                                objects: Some(eargs),
                            },
                        );
                        (SerInputType::sel, Some(mc.as_union_value()))
                    }
                };

                let c = InputRequest::create(
                    builder,
                    &InputRequestArgs {
                        client_from: *client_from,
                        input_msg_type,
                        input_msg,
                    },
                );
                (Message::ireq, Some(c.as_union_value()))
            }
            PacketMessage::SendInputResponse(client_from, client_ack) => {
                let c = InputResponse::create(
                    builder,
                    &InputResponseArgs {
                        client_from: *client_from,
                        client_ack: *client_ack,
                    },
                );
                (Message::ires, Some(c.as_union_value()))
            }
            PacketMessage::Invalid => panic!("wtf creating an invalid package? why? "),
            _ => unimplemented!(),
        }
    }

    pub fn to_flatbuffers<'a, 'b>(
        &'a self,
        builder: &'b mut flatbuffers::FlatBufferBuilder,
    ) -> flatbuffers::WIPOffset<NetPacket<'b>> {
        let (mtype, mdata) = self.type_to_flatbuffer(builder);
        let n = NetPacket::create(
            builder,
            &NetPacketArgs {
                tick: self.tick,
                source_client: self.source_client,
                dest_client: self.dest_client,
                timestamp: self.timestamp,
                id: self.message_id,
                message_type: mtype,
                message: mdata,
            },
        );
        builder.finish(n, None);
        n
    }
}

/// Create the network packet
///
/// It goes like this:
///
/// | Field        | Size | Description                               |
/// | ------------ | ---- | ----------------------------------------- |
/// | magic        | 4    | The magic header for the message          |
/// | flags        | 4    | Some flags for the message. Currently 0   |
/// | checksum     | 4    | The CRC32 checksum of the whole message   |
/// | payloadsize  | 4    | Size of the payload, in bytes             |
/// | payload      | n    | The payload                               |
///
/// Returns a vector with the binary data for the packet
pub fn create_packet(packet: Packet) -> Vec<u8> {
    let magic = "FAMI";

    let mut builder = flatbuffers::FlatBufferBuilder::new_with_capacity(1024);
    let npacket = packet.to_flatbuffers(&mut builder);
    let payload = builder.finished_data();
    let psize = payload.len();
    let mut res = Vec::new();

    res.extend(&magic.as_bytes().to_vec());
    res.extend(&vec![0, 0, 0, 0]); // flags
    res.extend(&vec![0, 0, 0, 0]); // checksum (to be filled later)
    res.extend(&vec![
        psize as u8 & 0xff,
        (psize >> 8) as u8 & 0xff,
        (psize >> 18) as u8 & 0xff,
        0,
    ]); // checksum (to be filled later)
    res.extend(payload);

    res
}

/// Decode the network packet
pub fn decode_packet(packet: &[u8]) -> Option<Packet> {
    if packet.len() <= 16 {
        return None;
    }

    let magic = match String::from_utf8(packet[0..4].to_vec()) {
        Ok(s) => {
            if s != "FAMI" {
                return None;
            } else {
                s
            }
        }
        Err(_) => return None, // Invalid message
    };
    let flags = packet[7] as u32 | ((packet[6] as u32) << 8) | ((packet[5] as u32) << 16);
    if flags != 0 {
        return None;
    }

    let checksum = packet[8] as u32
        | ((packet[9] as u32) << 8)
        | ((packet[10] as u32) << 16)
        | ((packet[11] as u32) << 24);
    if checksum == 0 {
        return None;
    }

    let psize = (packet[12] as u32
        | ((packet[13] as u32) << 8)
        | ((packet[14] as u32) << 16)
        | ((packet[15] as u32) << 24)) as usize;

    if psize > packet.len() {
        return None;
    }

    let payload = &packet[16..(psize - 16)];

    let npacket = get_root_as_net_packet(&payload);
    Some(Packet::from_flatbuffers(npacket))
}

/// Wait for the identification message
///
/// The first message of the client should be a StartRequest with its token.
///
/// If the message is not what we expect, return None. Else, return the token
/// TODO: maybe try waiting a few more times for the correct message?
pub async fn wait_for_identification(socket: &mut TcpStream) -> Option<String> {
    let mut size = [0u8; 512];
    let data = match socket.read(&mut size[..]).await {
        Ok(s) => {
            if s == 0 {
                return None;
            }
            size[0..s].to_vec()
        }
        Err(e) => {
            return None;
        }
    };

    match decode_packet(&data[..]) {
        Some(packet) => {
            if packet.tick > 0 {
                return None;
            }

            match packet.message {
                PacketMessage::StartRequest(id, token) => {
                    if id == packet.source_client {
                        Some(token)
                    } else {
                        None
                    }
                }
                _ => None,
            }
        }
        None => None,
    }
}

pub async fn handle_packet(packet: Packet, sender: &mut Sender<FMessage>, token: &str) {
    {
        println!(
            "Packet: {:?} (tick {}, source {}, dest {}, timestamp {}, id {}, type {:?})",
            packet,
            packet.tick,
            packet.source_client,
            packet.dest_client,
            packet.timestamp,
            packet.message_id,
            packet.message
        );
    }

    let npacket = packet.clone();
    match send_check_all_clients_connected_message(sender, token).await {
        Ok(true) => match send_push_game_packet_message(sender, token, packet).await {
            Ok(true) => match npacket.message {
                PacketMessage::LoadingRequest(percent) => {
                    send_push_game_packet_message(
                        sender,
                        token,
                        npacket
                            .make_server_own()
                            .to_new_client(npacket.source_client)
                            .to_new_message(PacketMessage::LoadingResponse(percent)),
                    )
                    .await
                    .unwrap();
                    send_push_game_packet_message(
                        sender,
                        token,
                        npacket
                            .to_new_client(0)
                            .to_new_message(PacketMessage::LoadingResponse(percent)),
                    )
                    .await
                    .unwrap();
                }
                PacketMessage::GameStartRequest => {
                    send_push_game_packet_message(
                        sender,
                        token,
                        npacket
                            .make_server_own()
                            .to_new_client(npacket.source_client)
                            .to_new_message(PacketMessage::GameStartResponse),
                    )
                    .await
                    .unwrap();
                    send_push_game_packet_message(
                        sender,
                        token,
                        npacket
                            .to_new_client(0)
                            .to_new_message(PacketMessage::GameStartResponse),
                    )
                    .await
                    .unwrap();
                }
                PacketMessage::SendInputRequest(_, _) => {
                    send_push_game_packet_message(sender, token, npacket.to_new_client(0))
                        .await
                        .unwrap();
                }
                _ => {}
            },
            _ => panic!("Error while sending packet!"),
        },
        Ok(false) => {
            // Only the initial packets can be sent without all clients
            // be connected.
            //
            // Return an error.
            println!("This is not the time for this packet yet!");
        }
        Err(_) => println!("Error while handling packet!"),
    }
}

/// Run the game server thread
///
/// Even if you connect, the game server will only accept your messages and answer them if
/// you call /connect and receive a positive answer
pub async fn run_game_server_thread(config: &ServerConfiguration, sender: Sender<FMessage>) {
    let listener = TcpListener::bind(format!("0.0.0.0:{}", config.gameport)).await;
    let mut listener = listener.unwrap();

    println!("Game server started at port {}", config.gameport);
    tokio::spawn(async move {
        while let Ok((mut socket, peer)) = listener.accept().await {
            let sender = sender.clone();
            tokio::spawn(async move {
                println!("Client Connected from: {}", peer.to_string());
                let mut sender = sender.clone();
                let (vtoken, token) = match wait_for_identification(&mut socket).await {
                    Some(t) => match send_connect_confirm_message(&mut sender, &t).await {
                        Ok(_) => (true, t),
                        Err(_) => (false, t),
                    },
                    None => {
                        println!("Invalid message received from client {}", peer.to_string());
                        (false, String::default())
                    }
                };

                println!("Client {} correctly identified", peer.to_string());

                let mut size = [0u8; 1024];
                let mut valid = vtoken;
                while valid {
                    let ready = socket
                        .ready(Interest::READABLE | Interest::WRITABLE)
                        .await
                        .unwrap();

                    if ready.is_readable() {
                        let data = match socket.read(&mut size[..]).await {
                            Ok(s) => {
                                if s == 0 {
                                    valid = false;
                                }
                                size[0..s].to_vec()
                            }
                            Err(e) => {
                                eprintln!("Error while reading: {:?}", e);
                                valid = false;
                                vec![0]
                            }
                        };
                        if !valid {
                            break;
                        }

                        match decode_packet(&data[..]) {
                            Some(packet) => {
                                handle_packet(packet, &mut sender, &token);
                            }
                            None => {
                                println!("Invalid packet of size {}", data.len());
                            }
                        };
                    }

                    if ready.is_writable() {
                        let packet = match send_pop_game_packet_message(&mut sender, &token).await {
                            Ok(p) => p,
                            Err(_) => panic!("unexpected error!"),
                        };

                        match packet {
                            Some(pkt) => {
                                let pdata = create_packet(pkt);
                                match socket.write(&pdata).await {
                                    Ok(s) => {
                                        if s <= 0 {
                                            valid = false;
                                        }
                                        ()
                                    }
                                    Err(e) => {
                                        eprintln!("Error while writing: {:?}", e);
                                        valid = false;
                                        ()
                                    }
                                }
                            }
                            None => {
                                tokio::time::sleep(Duration::from_millis(1000)).await;
                            }
                        }
                    }
                }
                println!("Client disconnected from: {}", peer.to_string());
            });
        }
    });
}
