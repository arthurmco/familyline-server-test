// automatically generated by the FlatBuffers compiler, do not modify



use std::mem;
use std::cmp::Ordering;

extern crate flatbuffers;
use self::flatbuffers::EndianScalar;

#[allow(non_camel_case_types)]
#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Message {
  NONE = 0,
  sreq = 1,
  sres = 2,
  lreq = 3,
  lres = 4,
  greq = 5,
  gres = 6,

}

pub const ENUM_MIN_MESSAGE: u8 = 0;
pub const ENUM_MAX_MESSAGE: u8 = 6;

impl<'a> flatbuffers::Follow<'a> for Message {
  type Inner = Self;
  #[inline]
  fn follow(buf: &'a [u8], loc: usize) -> Self::Inner {
    flatbuffers::read_scalar_at::<Self>(buf, loc)
  }
}

impl flatbuffers::EndianScalar for Message {
  #[inline]
  fn to_little_endian(self) -> Self {
    let n = u8::to_le(self as u8);
    let p = &n as *const u8 as *const Message;
    unsafe { *p }
  }
  #[inline]
  fn from_little_endian(self) -> Self {
    let n = u8::from_le(self as u8);
    let p = &n as *const u8 as *const Message;
    unsafe { *p }
  }
}

impl flatbuffers::Push for Message {
    type Output = Message;
    #[inline]
    fn push(&self, dst: &mut [u8], _rest: &[u8]) {
        flatbuffers::emplace_scalar::<Message>(dst, *self);
    }
}

#[allow(non_camel_case_types)]
pub const ENUM_VALUES_MESSAGE:[Message; 7] = [
  Message::NONE,
  Message::sreq,
  Message::sres,
  Message::lreq,
  Message::lres,
  Message::greq,
  Message::gres
];

#[allow(non_camel_case_types)]
pub const ENUM_NAMES_MESSAGE:[&'static str; 7] = [
    "NONE",
    "sreq",
    "sres",
    "lreq",
    "lres",
    "greq",
    "gres"
];

pub fn enum_name_message(e: Message) -> &'static str {
  let index = e as u8;
  ENUM_NAMES_MESSAGE[index as usize]
}

pub struct MessageUnionTableOffset {}
pub enum StartRequestOffset {}
#[derive(Copy, Clone, Debug, PartialEq)]

pub struct StartRequest<'a> {
  pub _tab: flatbuffers::Table<'a>,
}

impl<'a> flatbuffers::Follow<'a> for StartRequest<'a> {
    type Inner = StartRequest<'a>;
    #[inline]
    fn follow(buf: &'a [u8], loc: usize) -> Self::Inner {
        Self {
            _tab: flatbuffers::Table { buf: buf, loc: loc },
        }
    }
}

impl<'a> StartRequest<'a> {
    #[inline]
    pub fn init_from_table(table: flatbuffers::Table<'a>) -> Self {
        StartRequest {
            _tab: table,
        }
    }
    #[allow(unused_mut)]
    pub fn create<'bldr: 'args, 'args: 'mut_bldr, 'mut_bldr>(
        _fbb: &'mut_bldr mut flatbuffers::FlatBufferBuilder<'bldr>,
        args: &'args StartRequestArgs<'args>) -> flatbuffers::WIPOffset<StartRequest<'bldr>> {
      let mut builder = StartRequestBuilder::new(_fbb);
      builder.add_client_id(args.client_id);
      if let Some(x) = args.token { builder.add_token(x); }
      builder.finish()
    }

    pub const VT_CLIENT_ID: flatbuffers::VOffsetT = 4;
    pub const VT_TOKEN: flatbuffers::VOffsetT = 6;

  #[inline]
  pub fn client_id(&self) -> u64 {
    self._tab.get::<u64>(StartRequest::VT_CLIENT_ID, Some(0)).unwrap()
  }
  #[inline]
  pub fn token(&self) -> Option<&'a str> {
    self._tab.get::<flatbuffers::ForwardsUOffset<&str>>(StartRequest::VT_TOKEN, None)
  }
}

pub struct StartRequestArgs<'a> {
    pub client_id: u64,
    pub token: Option<flatbuffers::WIPOffset<&'a  str>>,
}
impl<'a> Default for StartRequestArgs<'a> {
    #[inline]
    fn default() -> Self {
        StartRequestArgs {
            client_id: 0,
            token: None,
        }
    }
}
pub struct StartRequestBuilder<'a: 'b, 'b> {
  fbb_: &'b mut flatbuffers::FlatBufferBuilder<'a>,
  start_: flatbuffers::WIPOffset<flatbuffers::TableUnfinishedWIPOffset>,
}
impl<'a: 'b, 'b> StartRequestBuilder<'a, 'b> {
  #[inline]
  pub fn add_client_id(&mut self, client_id: u64) {
    self.fbb_.push_slot::<u64>(StartRequest::VT_CLIENT_ID, client_id, 0);
  }
  #[inline]
  pub fn add_token(&mut self, token: flatbuffers::WIPOffset<&'b  str>) {
    self.fbb_.push_slot_always::<flatbuffers::WIPOffset<_>>(StartRequest::VT_TOKEN, token);
  }
  #[inline]
  pub fn new(_fbb: &'b mut flatbuffers::FlatBufferBuilder<'a>) -> StartRequestBuilder<'a, 'b> {
    let start = _fbb.start_table();
    StartRequestBuilder {
      fbb_: _fbb,
      start_: start,
    }
  }
  #[inline]
  pub fn finish(self) -> flatbuffers::WIPOffset<StartRequest<'a>> {
    let o = self.fbb_.end_table(self.start_);
    flatbuffers::WIPOffset::new(o.value())
  }
}

pub enum StartResponseOffset {}
#[derive(Copy, Clone, Debug, PartialEq)]

pub struct StartResponse<'a> {
  pub _tab: flatbuffers::Table<'a>,
}

impl<'a> flatbuffers::Follow<'a> for StartResponse<'a> {
    type Inner = StartResponse<'a>;
    #[inline]
    fn follow(buf: &'a [u8], loc: usize) -> Self::Inner {
        Self {
            _tab: flatbuffers::Table { buf: buf, loc: loc },
        }
    }
}

impl<'a> StartResponse<'a> {
    #[inline]
    pub fn init_from_table(table: flatbuffers::Table<'a>) -> Self {
        StartResponse {
            _tab: table,
        }
    }
    #[allow(unused_mut)]
    pub fn create<'bldr: 'args, 'args: 'mut_bldr, 'mut_bldr>(
        _fbb: &'mut_bldr mut flatbuffers::FlatBufferBuilder<'bldr>,
        args: &'args StartResponseArgs) -> flatbuffers::WIPOffset<StartResponse<'bldr>> {
      let mut builder = StartResponseBuilder::new(_fbb);
      builder.add_client_ack(args.client_ack);
      builder.add_all_clients_ack(args.all_clients_ack);
      builder.finish()
    }

    pub const VT_CLIENT_ACK: flatbuffers::VOffsetT = 4;
    pub const VT_ALL_CLIENTS_ACK: flatbuffers::VOffsetT = 6;

  #[inline]
  pub fn client_ack(&self) -> u64 {
    self._tab.get::<u64>(StartResponse::VT_CLIENT_ACK, Some(0)).unwrap()
  }
  #[inline]
  pub fn all_clients_ack(&self) -> bool {
    self._tab.get::<bool>(StartResponse::VT_ALL_CLIENTS_ACK, Some(false)).unwrap()
  }
}

pub struct StartResponseArgs {
    pub client_ack: u64,
    pub all_clients_ack: bool,
}
impl<'a> Default for StartResponseArgs {
    #[inline]
    fn default() -> Self {
        StartResponseArgs {
            client_ack: 0,
            all_clients_ack: false,
        }
    }
}
pub struct StartResponseBuilder<'a: 'b, 'b> {
  fbb_: &'b mut flatbuffers::FlatBufferBuilder<'a>,
  start_: flatbuffers::WIPOffset<flatbuffers::TableUnfinishedWIPOffset>,
}
impl<'a: 'b, 'b> StartResponseBuilder<'a, 'b> {
  #[inline]
  pub fn add_client_ack(&mut self, client_ack: u64) {
    self.fbb_.push_slot::<u64>(StartResponse::VT_CLIENT_ACK, client_ack, 0);
  }
  #[inline]
  pub fn add_all_clients_ack(&mut self, all_clients_ack: bool) {
    self.fbb_.push_slot::<bool>(StartResponse::VT_ALL_CLIENTS_ACK, all_clients_ack, false);
  }
  #[inline]
  pub fn new(_fbb: &'b mut flatbuffers::FlatBufferBuilder<'a>) -> StartResponseBuilder<'a, 'b> {
    let start = _fbb.start_table();
    StartResponseBuilder {
      fbb_: _fbb,
      start_: start,
    }
  }
  #[inline]
  pub fn finish(self) -> flatbuffers::WIPOffset<StartResponse<'a>> {
    let o = self.fbb_.end_table(self.start_);
    flatbuffers::WIPOffset::new(o.value())
  }
}

pub enum LoadingRequestOffset {}
#[derive(Copy, Clone, Debug, PartialEq)]

pub struct LoadingRequest<'a> {
  pub _tab: flatbuffers::Table<'a>,
}

impl<'a> flatbuffers::Follow<'a> for LoadingRequest<'a> {
    type Inner = LoadingRequest<'a>;
    #[inline]
    fn follow(buf: &'a [u8], loc: usize) -> Self::Inner {
        Self {
            _tab: flatbuffers::Table { buf: buf, loc: loc },
        }
    }
}

impl<'a> LoadingRequest<'a> {
    #[inline]
    pub fn init_from_table(table: flatbuffers::Table<'a>) -> Self {
        LoadingRequest {
            _tab: table,
        }
    }
    #[allow(unused_mut)]
    pub fn create<'bldr: 'args, 'args: 'mut_bldr, 'mut_bldr>(
        _fbb: &'mut_bldr mut flatbuffers::FlatBufferBuilder<'bldr>,
        args: &'args LoadingRequestArgs) -> flatbuffers::WIPOffset<LoadingRequest<'bldr>> {
      let mut builder = LoadingRequestBuilder::new(_fbb);
      builder.add_percent(args.percent);
      builder.finish()
    }

    pub const VT_PERCENT: flatbuffers::VOffsetT = 4;

  #[inline]
  pub fn percent(&self) -> u16 {
    self._tab.get::<u16>(LoadingRequest::VT_PERCENT, Some(0)).unwrap()
  }
}

pub struct LoadingRequestArgs {
    pub percent: u16,
}
impl<'a> Default for LoadingRequestArgs {
    #[inline]
    fn default() -> Self {
        LoadingRequestArgs {
            percent: 0,
        }
    }
}
pub struct LoadingRequestBuilder<'a: 'b, 'b> {
  fbb_: &'b mut flatbuffers::FlatBufferBuilder<'a>,
  start_: flatbuffers::WIPOffset<flatbuffers::TableUnfinishedWIPOffset>,
}
impl<'a: 'b, 'b> LoadingRequestBuilder<'a, 'b> {
  #[inline]
  pub fn add_percent(&mut self, percent: u16) {
    self.fbb_.push_slot::<u16>(LoadingRequest::VT_PERCENT, percent, 0);
  }
  #[inline]
  pub fn new(_fbb: &'b mut flatbuffers::FlatBufferBuilder<'a>) -> LoadingRequestBuilder<'a, 'b> {
    let start = _fbb.start_table();
    LoadingRequestBuilder {
      fbb_: _fbb,
      start_: start,
    }
  }
  #[inline]
  pub fn finish(self) -> flatbuffers::WIPOffset<LoadingRequest<'a>> {
    let o = self.fbb_.end_table(self.start_);
    flatbuffers::WIPOffset::new(o.value())
  }
}

pub enum LoadingResponseOffset {}
#[derive(Copy, Clone, Debug, PartialEq)]

pub struct LoadingResponse<'a> {
  pub _tab: flatbuffers::Table<'a>,
}

impl<'a> flatbuffers::Follow<'a> for LoadingResponse<'a> {
    type Inner = LoadingResponse<'a>;
    #[inline]
    fn follow(buf: &'a [u8], loc: usize) -> Self::Inner {
        Self {
            _tab: flatbuffers::Table { buf: buf, loc: loc },
        }
    }
}

impl<'a> LoadingResponse<'a> {
    #[inline]
    pub fn init_from_table(table: flatbuffers::Table<'a>) -> Self {
        LoadingResponse {
            _tab: table,
        }
    }
    #[allow(unused_mut)]
    pub fn create<'bldr: 'args, 'args: 'mut_bldr, 'mut_bldr>(
        _fbb: &'mut_bldr mut flatbuffers::FlatBufferBuilder<'bldr>,
        args: &'args LoadingResponseArgs) -> flatbuffers::WIPOffset<LoadingResponse<'bldr>> {
      let mut builder = LoadingResponseBuilder::new(_fbb);
      builder.add_percent(args.percent);
      builder.finish()
    }

    pub const VT_PERCENT: flatbuffers::VOffsetT = 4;

  #[inline]
  pub fn percent(&self) -> u16 {
    self._tab.get::<u16>(LoadingResponse::VT_PERCENT, Some(0)).unwrap()
  }
}

pub struct LoadingResponseArgs {
    pub percent: u16,
}
impl<'a> Default for LoadingResponseArgs {
    #[inline]
    fn default() -> Self {
        LoadingResponseArgs {
            percent: 0,
        }
    }
}
pub struct LoadingResponseBuilder<'a: 'b, 'b> {
  fbb_: &'b mut flatbuffers::FlatBufferBuilder<'a>,
  start_: flatbuffers::WIPOffset<flatbuffers::TableUnfinishedWIPOffset>,
}
impl<'a: 'b, 'b> LoadingResponseBuilder<'a, 'b> {
  #[inline]
  pub fn add_percent(&mut self, percent: u16) {
    self.fbb_.push_slot::<u16>(LoadingResponse::VT_PERCENT, percent, 0);
  }
  #[inline]
  pub fn new(_fbb: &'b mut flatbuffers::FlatBufferBuilder<'a>) -> LoadingResponseBuilder<'a, 'b> {
    let start = _fbb.start_table();
    LoadingResponseBuilder {
      fbb_: _fbb,
      start_: start,
    }
  }
  #[inline]
  pub fn finish(self) -> flatbuffers::WIPOffset<LoadingResponse<'a>> {
    let o = self.fbb_.end_table(self.start_);
    flatbuffers::WIPOffset::new(o.value())
  }
}

pub enum GameStartRequestOffset {}
#[derive(Copy, Clone, Debug, PartialEq)]

pub struct GameStartRequest<'a> {
  pub _tab: flatbuffers::Table<'a>,
}

impl<'a> flatbuffers::Follow<'a> for GameStartRequest<'a> {
    type Inner = GameStartRequest<'a>;
    #[inline]
    fn follow(buf: &'a [u8], loc: usize) -> Self::Inner {
        Self {
            _tab: flatbuffers::Table { buf: buf, loc: loc },
        }
    }
}

impl<'a> GameStartRequest<'a> {
    #[inline]
    pub fn init_from_table(table: flatbuffers::Table<'a>) -> Self {
        GameStartRequest {
            _tab: table,
        }
    }
    #[allow(unused_mut)]
    pub fn create<'bldr: 'args, 'args: 'mut_bldr, 'mut_bldr>(
        _fbb: &'mut_bldr mut flatbuffers::FlatBufferBuilder<'bldr>,
        args: &'args GameStartRequestArgs) -> flatbuffers::WIPOffset<GameStartRequest<'bldr>> {
      let mut builder = GameStartRequestBuilder::new(_fbb);
      builder.add_reserved(args.reserved);
      builder.finish()
    }

    pub const VT_RESERVED: flatbuffers::VOffsetT = 4;

  #[inline]
  pub fn reserved(&self) -> u32 {
    self._tab.get::<u32>(GameStartRequest::VT_RESERVED, Some(0)).unwrap()
  }
}

pub struct GameStartRequestArgs {
    pub reserved: u32,
}
impl<'a> Default for GameStartRequestArgs {
    #[inline]
    fn default() -> Self {
        GameStartRequestArgs {
            reserved: 0,
        }
    }
}
pub struct GameStartRequestBuilder<'a: 'b, 'b> {
  fbb_: &'b mut flatbuffers::FlatBufferBuilder<'a>,
  start_: flatbuffers::WIPOffset<flatbuffers::TableUnfinishedWIPOffset>,
}
impl<'a: 'b, 'b> GameStartRequestBuilder<'a, 'b> {
  #[inline]
  pub fn add_reserved(&mut self, reserved: u32) {
    self.fbb_.push_slot::<u32>(GameStartRequest::VT_RESERVED, reserved, 0);
  }
  #[inline]
  pub fn new(_fbb: &'b mut flatbuffers::FlatBufferBuilder<'a>) -> GameStartRequestBuilder<'a, 'b> {
    let start = _fbb.start_table();
    GameStartRequestBuilder {
      fbb_: _fbb,
      start_: start,
    }
  }
  #[inline]
  pub fn finish(self) -> flatbuffers::WIPOffset<GameStartRequest<'a>> {
    let o = self.fbb_.end_table(self.start_);
    flatbuffers::WIPOffset::new(o.value())
  }
}

pub enum GameStartResponseOffset {}
#[derive(Copy, Clone, Debug, PartialEq)]

pub struct GameStartResponse<'a> {
  pub _tab: flatbuffers::Table<'a>,
}

impl<'a> flatbuffers::Follow<'a> for GameStartResponse<'a> {
    type Inner = GameStartResponse<'a>;
    #[inline]
    fn follow(buf: &'a [u8], loc: usize) -> Self::Inner {
        Self {
            _tab: flatbuffers::Table { buf: buf, loc: loc },
        }
    }
}

impl<'a> GameStartResponse<'a> {
    #[inline]
    pub fn init_from_table(table: flatbuffers::Table<'a>) -> Self {
        GameStartResponse {
            _tab: table,
        }
    }
    #[allow(unused_mut)]
    pub fn create<'bldr: 'args, 'args: 'mut_bldr, 'mut_bldr>(
        _fbb: &'mut_bldr mut flatbuffers::FlatBufferBuilder<'bldr>,
        args: &'args GameStartResponseArgs) -> flatbuffers::WIPOffset<GameStartResponse<'bldr>> {
      let mut builder = GameStartResponseBuilder::new(_fbb);
      builder.add_reserved(args.reserved);
      builder.finish()
    }

    pub const VT_RESERVED: flatbuffers::VOffsetT = 4;

  #[inline]
  pub fn reserved(&self) -> u32 {
    self._tab.get::<u32>(GameStartResponse::VT_RESERVED, Some(0)).unwrap()
  }
}

pub struct GameStartResponseArgs {
    pub reserved: u32,
}
impl<'a> Default for GameStartResponseArgs {
    #[inline]
    fn default() -> Self {
        GameStartResponseArgs {
            reserved: 0,
        }
    }
}
pub struct GameStartResponseBuilder<'a: 'b, 'b> {
  fbb_: &'b mut flatbuffers::FlatBufferBuilder<'a>,
  start_: flatbuffers::WIPOffset<flatbuffers::TableUnfinishedWIPOffset>,
}
impl<'a: 'b, 'b> GameStartResponseBuilder<'a, 'b> {
  #[inline]
  pub fn add_reserved(&mut self, reserved: u32) {
    self.fbb_.push_slot::<u32>(GameStartResponse::VT_RESERVED, reserved, 0);
  }
  #[inline]
  pub fn new(_fbb: &'b mut flatbuffers::FlatBufferBuilder<'a>) -> GameStartResponseBuilder<'a, 'b> {
    let start = _fbb.start_table();
    GameStartResponseBuilder {
      fbb_: _fbb,
      start_: start,
    }
  }
  #[inline]
  pub fn finish(self) -> flatbuffers::WIPOffset<GameStartResponse<'a>> {
    let o = self.fbb_.end_table(self.start_);
    flatbuffers::WIPOffset::new(o.value())
  }
}

pub enum NetPacketOffset {}
#[derive(Copy, Clone, Debug, PartialEq)]

pub struct NetPacket<'a> {
  pub _tab: flatbuffers::Table<'a>,
}

impl<'a> flatbuffers::Follow<'a> for NetPacket<'a> {
    type Inner = NetPacket<'a>;
    #[inline]
    fn follow(buf: &'a [u8], loc: usize) -> Self::Inner {
        Self {
            _tab: flatbuffers::Table { buf: buf, loc: loc },
        }
    }
}

impl<'a> NetPacket<'a> {
    #[inline]
    pub fn init_from_table(table: flatbuffers::Table<'a>) -> Self {
        NetPacket {
            _tab: table,
        }
    }
    #[allow(unused_mut)]
    pub fn create<'bldr: 'args, 'args: 'mut_bldr, 'mut_bldr>(
        _fbb: &'mut_bldr mut flatbuffers::FlatBufferBuilder<'bldr>,
        args: &'args NetPacketArgs) -> flatbuffers::WIPOffset<NetPacket<'bldr>> {
      let mut builder = NetPacketBuilder::new(_fbb);
      builder.add_id(args.id);
      builder.add_timestamp(args.timestamp);
      builder.add_dest_client(args.dest_client);
      builder.add_source_client(args.source_client);
      builder.add_tick(args.tick);
      if let Some(x) = args.message { builder.add_message(x); }
      builder.add_message_type(args.message_type);
      builder.finish()
    }

    pub const VT_TICK: flatbuffers::VOffsetT = 4;
    pub const VT_SOURCE_CLIENT: flatbuffers::VOffsetT = 6;
    pub const VT_DEST_CLIENT: flatbuffers::VOffsetT = 8;
    pub const VT_TIMESTAMP: flatbuffers::VOffsetT = 10;
    pub const VT_ID: flatbuffers::VOffsetT = 12;
    pub const VT_MESSAGE_TYPE: flatbuffers::VOffsetT = 14;
    pub const VT_MESSAGE: flatbuffers::VOffsetT = 16;

  #[inline]
  pub fn tick(&self) -> u64 {
    self._tab.get::<u64>(NetPacket::VT_TICK, Some(0)).unwrap()
  }
  #[inline]
  pub fn source_client(&self) -> u64 {
    self._tab.get::<u64>(NetPacket::VT_SOURCE_CLIENT, Some(0)).unwrap()
  }
  #[inline]
  pub fn dest_client(&self) -> u64 {
    self._tab.get::<u64>(NetPacket::VT_DEST_CLIENT, Some(0)).unwrap()
  }
  #[inline]
  pub fn timestamp(&self) -> u64 {
    self._tab.get::<u64>(NetPacket::VT_TIMESTAMP, Some(0)).unwrap()
  }
  #[inline]
  pub fn id(&self) -> u64 {
    self._tab.get::<u64>(NetPacket::VT_ID, Some(0)).unwrap()
  }
  #[inline]
  pub fn message_type(&self) -> Message {
    self._tab.get::<Message>(NetPacket::VT_MESSAGE_TYPE, Some(Message::NONE)).unwrap()
  }
  #[inline]
  pub fn message(&self) -> flatbuffers::Table<'a> {
    self._tab.get::<flatbuffers::ForwardsUOffset<flatbuffers::Table<'a>>>(NetPacket::VT_MESSAGE, None).unwrap()
  }
  #[inline]
  #[allow(non_snake_case)]
  pub fn message_as_sreq(&self) -> Option<StartRequest<'a>> {
    if self.message_type() == Message::sreq {
      let u = self.message();
      Some(StartRequest::init_from_table(u))
    } else {
      None
    }
  }

  #[inline]
  #[allow(non_snake_case)]
  pub fn message_as_sres(&self) -> Option<StartResponse<'a>> {
    if self.message_type() == Message::sres {
      let u = self.message();
      Some(StartResponse::init_from_table(u))
    } else {
      None
    }
  }

  #[inline]
  #[allow(non_snake_case)]
  pub fn message_as_lreq(&self) -> Option<LoadingRequest<'a>> {
    if self.message_type() == Message::lreq {
      let u = self.message();
      Some(LoadingRequest::init_from_table(u))
    } else {
      None
    }
  }

  #[inline]
  #[allow(non_snake_case)]
  pub fn message_as_lres(&self) -> Option<LoadingResponse<'a>> {
    if self.message_type() == Message::lres {
      let u = self.message();
      Some(LoadingResponse::init_from_table(u))
    } else {
      None
    }
  }

  #[inline]
  #[allow(non_snake_case)]
  pub fn message_as_greq(&self) -> Option<GameStartRequest<'a>> {
    if self.message_type() == Message::greq {
      let u = self.message();
      Some(GameStartRequest::init_from_table(u))
    } else {
      None
    }
  }

  #[inline]
  #[allow(non_snake_case)]
  pub fn message_as_gres(&self) -> Option<GameStartResponse<'a>> {
    if self.message_type() == Message::gres {
      let u = self.message();
      Some(GameStartResponse::init_from_table(u))
    } else {
      None
    }
  }

}

pub struct NetPacketArgs {
    pub tick: u64,
    pub source_client: u64,
    pub dest_client: u64,
    pub timestamp: u64,
    pub id: u64,
    pub message_type: Message,
    pub message: Option<flatbuffers::WIPOffset<flatbuffers::UnionWIPOffset>>,
}
impl<'a> Default for NetPacketArgs {
    #[inline]
    fn default() -> Self {
        NetPacketArgs {
            tick: 0,
            source_client: 0,
            dest_client: 0,
            timestamp: 0,
            id: 0,
            message_type: Message::NONE,
            message: None, // required field
        }
    }
}
pub struct NetPacketBuilder<'a: 'b, 'b> {
  fbb_: &'b mut flatbuffers::FlatBufferBuilder<'a>,
  start_: flatbuffers::WIPOffset<flatbuffers::TableUnfinishedWIPOffset>,
}
impl<'a: 'b, 'b> NetPacketBuilder<'a, 'b> {
  #[inline]
  pub fn add_tick(&mut self, tick: u64) {
    self.fbb_.push_slot::<u64>(NetPacket::VT_TICK, tick, 0);
  }
  #[inline]
  pub fn add_source_client(&mut self, source_client: u64) {
    self.fbb_.push_slot::<u64>(NetPacket::VT_SOURCE_CLIENT, source_client, 0);
  }
  #[inline]
  pub fn add_dest_client(&mut self, dest_client: u64) {
    self.fbb_.push_slot::<u64>(NetPacket::VT_DEST_CLIENT, dest_client, 0);
  }
  #[inline]
  pub fn add_timestamp(&mut self, timestamp: u64) {
    self.fbb_.push_slot::<u64>(NetPacket::VT_TIMESTAMP, timestamp, 0);
  }
  #[inline]
  pub fn add_id(&mut self, id: u64) {
    self.fbb_.push_slot::<u64>(NetPacket::VT_ID, id, 0);
  }
  #[inline]
  pub fn add_message_type(&mut self, message_type: Message) {
    self.fbb_.push_slot::<Message>(NetPacket::VT_MESSAGE_TYPE, message_type, Message::NONE);
  }
  #[inline]
  pub fn add_message(&mut self, message: flatbuffers::WIPOffset<flatbuffers::UnionWIPOffset>) {
    self.fbb_.push_slot_always::<flatbuffers::WIPOffset<_>>(NetPacket::VT_MESSAGE, message);
  }
  #[inline]
  pub fn new(_fbb: &'b mut flatbuffers::FlatBufferBuilder<'a>) -> NetPacketBuilder<'a, 'b> {
    let start = _fbb.start_table();
    NetPacketBuilder {
      fbb_: _fbb,
      start_: start,
    }
  }
  #[inline]
  pub fn finish(self) -> flatbuffers::WIPOffset<NetPacket<'a>> {
    let o = self.fbb_.end_table(self.start_);
    self.fbb_.required(o, NetPacket::VT_MESSAGE,"message");
    flatbuffers::WIPOffset::new(o.value())
  }
}

#[inline]
pub fn get_root_as_net_packet<'a>(buf: &'a [u8]) -> NetPacket<'a> {
  flatbuffers::get_root::<NetPacket<'a>>(buf)
}

#[inline]
pub fn get_size_prefixed_root_as_net_packet<'a>(buf: &'a [u8]) -> NetPacket<'a> {
  flatbuffers::get_size_prefixed_root::<NetPacket<'a>>(buf)
}

#[inline]
pub fn finish_net_packet_buffer<'a, 'b>(
    fbb: &'b mut flatbuffers::FlatBufferBuilder<'a>,
    root: flatbuffers::WIPOffset<NetPacket<'a>>) {
  fbb.finish(root, None);
}

#[inline]
pub fn finish_size_prefixed_net_packet_buffer<'a, 'b>(fbb: &'b mut flatbuffers::FlatBufferBuilder<'a>, root: flatbuffers::WIPOffset<NetPacket<'a>>) {
  fbb.finish_size_prefixed(root, None);
}
