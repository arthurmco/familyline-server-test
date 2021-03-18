// automatically generated by the FlatBuffers compiler, do not modify

use std::cmp::Ordering;
use std::mem;

extern crate flatbuffers;
use self::flatbuffers::EndianScalar;

#[allow(non_camel_case_types)]
#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum InputType {
    NONE = 0,
    cmd = 1,
    sel = 2,
    obj_move = 3,
    cam_move = 4,
    cam_rotate = 5,
    create = 6,
}

pub const ENUM_MIN_INPUT_TYPE: u8 = 0;
pub const ENUM_MAX_INPUT_TYPE: u8 = 6;

impl<'a> flatbuffers::Follow<'a> for InputType {
    type Inner = Self;
    #[inline]
    fn follow(buf: &'a [u8], loc: usize) -> Self::Inner {
        flatbuffers::read_scalar_at::<Self>(buf, loc)
    }
}

impl flatbuffers::EndianScalar for InputType {
    #[inline]
    fn to_little_endian(self) -> Self {
        let n = u8::to_le(self as u8);
        let p = &n as *const u8 as *const InputType;
        unsafe { *p }
    }
    #[inline]
    fn from_little_endian(self) -> Self {
        let n = u8::from_le(self as u8);
        let p = &n as *const u8 as *const InputType;
        unsafe { *p }
    }
}

impl flatbuffers::Push for InputType {
    type Output = InputType;
    #[inline]
    fn push(&self, dst: &mut [u8], _rest: &[u8]) {
        flatbuffers::emplace_scalar::<InputType>(dst, *self);
    }
}

#[allow(non_camel_case_types)]
pub const ENUM_VALUES_INPUT_TYPE: [InputType; 7] = [
    InputType::NONE,
    InputType::cmd,
    InputType::sel,
    InputType::obj_move,
    InputType::cam_move,
    InputType::cam_rotate,
    InputType::create,
];

#[allow(non_camel_case_types)]
pub const ENUM_NAMES_INPUT_TYPE: [&'static str; 7] = [
    "NONE",
    "cmd",
    "sel",
    "obj_move",
    "cam_move",
    "cam_rotate",
    "create",
];

pub fn enum_name_input_type(e: InputType) -> &'static str {
    let index = e as u8;
    ENUM_NAMES_INPUT_TYPE[index as usize]
}

pub struct InputTypeUnionTableOffset {}
pub enum CommandInputAOffset {}
#[derive(Copy, Clone, Debug, PartialEq)]

pub struct CommandInputA<'a> {
    pub _tab: flatbuffers::Table<'a>,
}

impl<'a> flatbuffers::Follow<'a> for CommandInputA<'a> {
    type Inner = CommandInputA<'a>;
    #[inline]
    fn follow(buf: &'a [u8], loc: usize) -> Self::Inner {
        Self {
            _tab: flatbuffers::Table { buf: buf, loc: loc },
        }
    }
}

impl<'a> CommandInputA<'a> {
    #[inline]
    pub fn init_from_table(table: flatbuffers::Table<'a>) -> Self {
        CommandInputA { _tab: table }
    }
    #[allow(unused_mut)]
    pub fn create<'bldr: 'args, 'args: 'mut_bldr, 'mut_bldr>(
        _fbb: &'mut_bldr mut flatbuffers::FlatBufferBuilder<'bldr>,
        args: &'args CommandInputAArgs<'args>,
    ) -> flatbuffers::WIPOffset<CommandInputA<'bldr>> {
        let mut builder = CommandInputABuilder::new(_fbb);
        if let Some(x) = args.args {
            builder.add_args(x);
        }
        builder.finish()
    }

    pub const VT_ARGS: flatbuffers::VOffsetT = 4;

    #[inline]
    pub fn args(&self) -> Option<flatbuffers::Vector<'a, u64>> {
        self._tab
            .get::<flatbuffers::ForwardsUOffset<flatbuffers::Vector<'a, u64>>>(
                CommandInputA::VT_ARGS,
                None,
            )
    }
}

pub struct CommandInputAArgs<'a> {
    pub args: Option<flatbuffers::WIPOffset<flatbuffers::Vector<'a, u64>>>,
}
impl<'a> Default for CommandInputAArgs<'a> {
    #[inline]
    fn default() -> Self {
        CommandInputAArgs { args: None }
    }
}
pub struct CommandInputABuilder<'a: 'b, 'b> {
    fbb_: &'b mut flatbuffers::FlatBufferBuilder<'a>,
    start_: flatbuffers::WIPOffset<flatbuffers::TableUnfinishedWIPOffset>,
}
impl<'a: 'b, 'b> CommandInputABuilder<'a, 'b> {
    #[inline]
    pub fn add_args(&mut self, args: flatbuffers::WIPOffset<flatbuffers::Vector<'b, u64>>) {
        self.fbb_
            .push_slot_always::<flatbuffers::WIPOffset<_>>(CommandInputA::VT_ARGS, args);
    }
    #[inline]
    pub fn new(_fbb: &'b mut flatbuffers::FlatBufferBuilder<'a>) -> CommandInputABuilder<'a, 'b> {
        let start = _fbb.start_table();
        CommandInputABuilder {
            fbb_: _fbb,
            start_: start,
        }
    }
    #[inline]
    pub fn finish(self) -> flatbuffers::WIPOffset<CommandInputA<'a>> {
        let o = self.fbb_.end_table(self.start_);
        flatbuffers::WIPOffset::new(o.value())
    }
}

pub enum CommandInputOffset {}
#[derive(Copy, Clone, Debug, PartialEq)]

pub struct CommandInput<'a> {
    pub _tab: flatbuffers::Table<'a>,
}

impl<'a> flatbuffers::Follow<'a> for CommandInput<'a> {
    type Inner = CommandInput<'a>;
    #[inline]
    fn follow(buf: &'a [u8], loc: usize) -> Self::Inner {
        Self {
            _tab: flatbuffers::Table { buf: buf, loc: loc },
        }
    }
}

impl<'a> CommandInput<'a> {
    #[inline]
    pub fn init_from_table(table: flatbuffers::Table<'a>) -> Self {
        CommandInput { _tab: table }
    }
    #[allow(unused_mut)]
    pub fn create<'bldr: 'args, 'args: 'mut_bldr, 'mut_bldr>(
        _fbb: &'mut_bldr mut flatbuffers::FlatBufferBuilder<'bldr>,
        args: &'args CommandInputArgs<'args>,
    ) -> flatbuffers::WIPOffset<CommandInput<'bldr>> {
        let mut builder = CommandInputBuilder::new(_fbb);
        if let Some(x) = args.args {
            builder.add_args(x);
        }
        if let Some(x) = args.command {
            builder.add_command(x);
        }
        builder.finish()
    }

    pub const VT_COMMAND: flatbuffers::VOffsetT = 4;
    pub const VT_ARGS: flatbuffers::VOffsetT = 6;

    #[inline]
    pub fn command(&self) -> &'a str {
        self._tab
            .get::<flatbuffers::ForwardsUOffset<&str>>(CommandInput::VT_COMMAND, None)
            .unwrap()
    }
    #[inline]
    pub fn args(&self) -> Option<CommandInputA<'a>> {
        self._tab
            .get::<flatbuffers::ForwardsUOffset<CommandInputA<'a>>>(CommandInput::VT_ARGS, None)
    }
}

pub struct CommandInputArgs<'a> {
    pub command: Option<flatbuffers::WIPOffset<&'a str>>,
    pub args: Option<flatbuffers::WIPOffset<CommandInputA<'a>>>,
}
impl<'a> Default for CommandInputArgs<'a> {
    #[inline]
    fn default() -> Self {
        CommandInputArgs {
            command: None, // required field
            args: None,
        }
    }
}
pub struct CommandInputBuilder<'a: 'b, 'b> {
    fbb_: &'b mut flatbuffers::FlatBufferBuilder<'a>,
    start_: flatbuffers::WIPOffset<flatbuffers::TableUnfinishedWIPOffset>,
}
impl<'a: 'b, 'b> CommandInputBuilder<'a, 'b> {
    #[inline]
    pub fn add_command(&mut self, command: flatbuffers::WIPOffset<&'b str>) {
        self.fbb_
            .push_slot_always::<flatbuffers::WIPOffset<_>>(CommandInput::VT_COMMAND, command);
    }
    #[inline]
    pub fn add_args(&mut self, args: flatbuffers::WIPOffset<CommandInputA<'b>>) {
        self.fbb_
            .push_slot_always::<flatbuffers::WIPOffset<CommandInputA>>(CommandInput::VT_ARGS, args);
    }
    #[inline]
    pub fn new(_fbb: &'b mut flatbuffers::FlatBufferBuilder<'a>) -> CommandInputBuilder<'a, 'b> {
        let start = _fbb.start_table();
        CommandInputBuilder {
            fbb_: _fbb,
            start_: start,
        }
    }
    #[inline]
    pub fn finish(self) -> flatbuffers::WIPOffset<CommandInput<'a>> {
        let o = self.fbb_.end_table(self.start_);
        self.fbb_.required(o, CommandInput::VT_COMMAND, "command");
        flatbuffers::WIPOffset::new(o.value())
    }
}

pub enum SelectActionObjectsOffset {}
#[derive(Copy, Clone, Debug, PartialEq)]

pub struct SelectActionObjects<'a> {
    pub _tab: flatbuffers::Table<'a>,
}

impl<'a> flatbuffers::Follow<'a> for SelectActionObjects<'a> {
    type Inner = SelectActionObjects<'a>;
    #[inline]
    fn follow(buf: &'a [u8], loc: usize) -> Self::Inner {
        Self {
            _tab: flatbuffers::Table { buf: buf, loc: loc },
        }
    }
}

impl<'a> SelectActionObjects<'a> {
    #[inline]
    pub fn init_from_table(table: flatbuffers::Table<'a>) -> Self {
        SelectActionObjects { _tab: table }
    }
    #[allow(unused_mut)]
    pub fn create<'bldr: 'args, 'args: 'mut_bldr, 'mut_bldr>(
        _fbb: &'mut_bldr mut flatbuffers::FlatBufferBuilder<'bldr>,
        args: &'args SelectActionObjectsArgs<'args>,
    ) -> flatbuffers::WIPOffset<SelectActionObjects<'bldr>> {
        let mut builder = SelectActionObjectsBuilder::new(_fbb);
        if let Some(x) = args.values {
            builder.add_values(x);
        }
        builder.finish()
    }

    pub const VT_VALUES: flatbuffers::VOffsetT = 4;

    #[inline]
    pub fn values(&self) -> flatbuffers::Vector<'a, u64> {
        self._tab
            .get::<flatbuffers::ForwardsUOffset<flatbuffers::Vector<'a, u64>>>(
                SelectActionObjects::VT_VALUES,
                None,
            )
            .unwrap()
    }
}

pub struct SelectActionObjectsArgs<'a> {
    pub values: Option<flatbuffers::WIPOffset<flatbuffers::Vector<'a, u64>>>,
}
impl<'a> Default for SelectActionObjectsArgs<'a> {
    #[inline]
    fn default() -> Self {
        SelectActionObjectsArgs {
            values: None, // required field
        }
    }
}
pub struct SelectActionObjectsBuilder<'a: 'b, 'b> {
    fbb_: &'b mut flatbuffers::FlatBufferBuilder<'a>,
    start_: flatbuffers::WIPOffset<flatbuffers::TableUnfinishedWIPOffset>,
}
impl<'a: 'b, 'b> SelectActionObjectsBuilder<'a, 'b> {
    #[inline]
    pub fn add_values(&mut self, values: flatbuffers::WIPOffset<flatbuffers::Vector<'b, u64>>) {
        self.fbb_
            .push_slot_always::<flatbuffers::WIPOffset<_>>(SelectActionObjects::VT_VALUES, values);
    }
    #[inline]
    pub fn new(
        _fbb: &'b mut flatbuffers::FlatBufferBuilder<'a>,
    ) -> SelectActionObjectsBuilder<'a, 'b> {
        let start = _fbb.start_table();
        SelectActionObjectsBuilder {
            fbb_: _fbb,
            start_: start,
        }
    }
    #[inline]
    pub fn finish(self) -> flatbuffers::WIPOffset<SelectActionObjects<'a>> {
        let o = self.fbb_.end_table(self.start_);
        self.fbb_
            .required(o, SelectActionObjects::VT_VALUES, "values");
        flatbuffers::WIPOffset::new(o.value())
    }
}

pub enum SelectActionOffset {}
#[derive(Copy, Clone, Debug, PartialEq)]

pub struct SelectAction<'a> {
    pub _tab: flatbuffers::Table<'a>,
}

impl<'a> flatbuffers::Follow<'a> for SelectAction<'a> {
    type Inner = SelectAction<'a>;
    #[inline]
    fn follow(buf: &'a [u8], loc: usize) -> Self::Inner {
        Self {
            _tab: flatbuffers::Table { buf: buf, loc: loc },
        }
    }
}

impl<'a> SelectAction<'a> {
    #[inline]
    pub fn init_from_table(table: flatbuffers::Table<'a>) -> Self {
        SelectAction { _tab: table }
    }
    #[allow(unused_mut)]
    pub fn create<'bldr: 'args, 'args: 'mut_bldr, 'mut_bldr>(
        _fbb: &'mut_bldr mut flatbuffers::FlatBufferBuilder<'bldr>,
        args: &'args SelectActionArgs<'args>,
    ) -> flatbuffers::WIPOffset<SelectAction<'bldr>> {
        let mut builder = SelectActionBuilder::new(_fbb);
        if let Some(x) = args.objects {
            builder.add_objects(x);
        }
        builder.finish()
    }

    pub const VT_OBJECTS: flatbuffers::VOffsetT = 4;

    #[inline]
    pub fn objects(&self) -> SelectActionObjects<'a> {
        self._tab
            .get::<flatbuffers::ForwardsUOffset<SelectActionObjects<'a>>>(
                SelectAction::VT_OBJECTS,
                None,
            )
            .unwrap()
    }
}

pub struct SelectActionArgs<'a> {
    pub objects: Option<flatbuffers::WIPOffset<SelectActionObjects<'a>>>,
}
impl<'a> Default for SelectActionArgs<'a> {
    #[inline]
    fn default() -> Self {
        SelectActionArgs {
            objects: None, // required field
        }
    }
}
pub struct SelectActionBuilder<'a: 'b, 'b> {
    fbb_: &'b mut flatbuffers::FlatBufferBuilder<'a>,
    start_: flatbuffers::WIPOffset<flatbuffers::TableUnfinishedWIPOffset>,
}
impl<'a: 'b, 'b> SelectActionBuilder<'a, 'b> {
    #[inline]
    pub fn add_objects(&mut self, objects: flatbuffers::WIPOffset<SelectActionObjects<'b>>) {
        self.fbb_
            .push_slot_always::<flatbuffers::WIPOffset<SelectActionObjects>>(
                SelectAction::VT_OBJECTS,
                objects,
            );
    }
    #[inline]
    pub fn new(_fbb: &'b mut flatbuffers::FlatBufferBuilder<'a>) -> SelectActionBuilder<'a, 'b> {
        let start = _fbb.start_table();
        SelectActionBuilder {
            fbb_: _fbb,
            start_: start,
        }
    }
    #[inline]
    pub fn finish(self) -> flatbuffers::WIPOffset<SelectAction<'a>> {
        let o = self.fbb_.end_table(self.start_);
        self.fbb_.required(o, SelectAction::VT_OBJECTS, "objects");
        flatbuffers::WIPOffset::new(o.value())
    }
}

pub enum ObjectMoveOffset {}
#[derive(Copy, Clone, Debug, PartialEq)]

pub struct ObjectMove<'a> {
    pub _tab: flatbuffers::Table<'a>,
}

impl<'a> flatbuffers::Follow<'a> for ObjectMove<'a> {
    type Inner = ObjectMove<'a>;
    #[inline]
    fn follow(buf: &'a [u8], loc: usize) -> Self::Inner {
        Self {
            _tab: flatbuffers::Table { buf: buf, loc: loc },
        }
    }
}

impl<'a> ObjectMove<'a> {
    #[inline]
    pub fn init_from_table(table: flatbuffers::Table<'a>) -> Self {
        ObjectMove { _tab: table }
    }
    #[allow(unused_mut)]
    pub fn create<'bldr: 'args, 'args: 'mut_bldr, 'mut_bldr>(
        _fbb: &'mut_bldr mut flatbuffers::FlatBufferBuilder<'bldr>,
        args: &'args ObjectMoveArgs,
    ) -> flatbuffers::WIPOffset<ObjectMove<'bldr>> {
        let mut builder = ObjectMoveBuilder::new(_fbb);
        builder.add_y_pos(args.y_pos);
        builder.add_x_pos(args.x_pos);
        builder.finish()
    }

    pub const VT_X_POS: flatbuffers::VOffsetT = 4;
    pub const VT_Y_POS: flatbuffers::VOffsetT = 6;

    #[inline]
    pub fn x_pos(&self) -> u32 {
        self._tab.get::<u32>(ObjectMove::VT_X_POS, Some(0)).unwrap()
    }
    #[inline]
    pub fn y_pos(&self) -> u32 {
        self._tab.get::<u32>(ObjectMove::VT_Y_POS, Some(0)).unwrap()
    }
}

pub struct ObjectMoveArgs {
    pub x_pos: u32,
    pub y_pos: u32,
}
impl<'a> Default for ObjectMoveArgs {
    #[inline]
    fn default() -> Self {
        ObjectMoveArgs { x_pos: 0, y_pos: 0 }
    }
}
pub struct ObjectMoveBuilder<'a: 'b, 'b> {
    fbb_: &'b mut flatbuffers::FlatBufferBuilder<'a>,
    start_: flatbuffers::WIPOffset<flatbuffers::TableUnfinishedWIPOffset>,
}
impl<'a: 'b, 'b> ObjectMoveBuilder<'a, 'b> {
    #[inline]
    pub fn add_x_pos(&mut self, x_pos: u32) {
        self.fbb_.push_slot::<u32>(ObjectMove::VT_X_POS, x_pos, 0);
    }
    #[inline]
    pub fn add_y_pos(&mut self, y_pos: u32) {
        self.fbb_.push_slot::<u32>(ObjectMove::VT_Y_POS, y_pos, 0);
    }
    #[inline]
    pub fn new(_fbb: &'b mut flatbuffers::FlatBufferBuilder<'a>) -> ObjectMoveBuilder<'a, 'b> {
        let start = _fbb.start_table();
        ObjectMoveBuilder {
            fbb_: _fbb,
            start_: start,
        }
    }
    #[inline]
    pub fn finish(self) -> flatbuffers::WIPOffset<ObjectMove<'a>> {
        let o = self.fbb_.end_table(self.start_);
        flatbuffers::WIPOffset::new(o.value())
    }
}

pub enum CameraMoveOffset {}
#[derive(Copy, Clone, Debug, PartialEq)]

pub struct CameraMove<'a> {
    pub _tab: flatbuffers::Table<'a>,
}

impl<'a> flatbuffers::Follow<'a> for CameraMove<'a> {
    type Inner = CameraMove<'a>;
    #[inline]
    fn follow(buf: &'a [u8], loc: usize) -> Self::Inner {
        Self {
            _tab: flatbuffers::Table { buf: buf, loc: loc },
        }
    }
}

impl<'a> CameraMove<'a> {
    #[inline]
    pub fn init_from_table(table: flatbuffers::Table<'a>) -> Self {
        CameraMove { _tab: table }
    }
    #[allow(unused_mut)]
    pub fn create<'bldr: 'args, 'args: 'mut_bldr, 'mut_bldr>(
        _fbb: &'mut_bldr mut flatbuffers::FlatBufferBuilder<'bldr>,
        args: &'args CameraMoveArgs,
    ) -> flatbuffers::WIPOffset<CameraMove<'bldr>> {
        let mut builder = CameraMoveBuilder::new(_fbb);
        builder.add_zoom_delta(args.zoom_delta);
        builder.add_y_delta(args.y_delta);
        builder.add_x_delta(args.x_delta);
        builder.finish()
    }

    pub const VT_X_DELTA: flatbuffers::VOffsetT = 4;
    pub const VT_Y_DELTA: flatbuffers::VOffsetT = 6;
    pub const VT_ZOOM_DELTA: flatbuffers::VOffsetT = 8;

    #[inline]
    pub fn x_delta(&self) -> f64 {
        self._tab
            .get::<f64>(CameraMove::VT_X_DELTA, Some(0.0))
            .unwrap()
    }
    #[inline]
    pub fn y_delta(&self) -> f64 {
        self._tab
            .get::<f64>(CameraMove::VT_Y_DELTA, Some(0.0))
            .unwrap()
    }
    #[inline]
    pub fn zoom_delta(&self) -> f64 {
        self._tab
            .get::<f64>(CameraMove::VT_ZOOM_DELTA, Some(0.0))
            .unwrap()
    }
}

pub struct CameraMoveArgs {
    pub x_delta: f64,
    pub y_delta: f64,
    pub zoom_delta: f64,
}
impl<'a> Default for CameraMoveArgs {
    #[inline]
    fn default() -> Self {
        CameraMoveArgs {
            x_delta: 0.0,
            y_delta: 0.0,
            zoom_delta: 0.0,
        }
    }
}
pub struct CameraMoveBuilder<'a: 'b, 'b> {
    fbb_: &'b mut flatbuffers::FlatBufferBuilder<'a>,
    start_: flatbuffers::WIPOffset<flatbuffers::TableUnfinishedWIPOffset>,
}
impl<'a: 'b, 'b> CameraMoveBuilder<'a, 'b> {
    #[inline]
    pub fn add_x_delta(&mut self, x_delta: f64) {
        self.fbb_
            .push_slot::<f64>(CameraMove::VT_X_DELTA, x_delta, 0.0);
    }
    #[inline]
    pub fn add_y_delta(&mut self, y_delta: f64) {
        self.fbb_
            .push_slot::<f64>(CameraMove::VT_Y_DELTA, y_delta, 0.0);
    }
    #[inline]
    pub fn add_zoom_delta(&mut self, zoom_delta: f64) {
        self.fbb_
            .push_slot::<f64>(CameraMove::VT_ZOOM_DELTA, zoom_delta, 0.0);
    }
    #[inline]
    pub fn new(_fbb: &'b mut flatbuffers::FlatBufferBuilder<'a>) -> CameraMoveBuilder<'a, 'b> {
        let start = _fbb.start_table();
        CameraMoveBuilder {
            fbb_: _fbb,
            start_: start,
        }
    }
    #[inline]
    pub fn finish(self) -> flatbuffers::WIPOffset<CameraMove<'a>> {
        let o = self.fbb_.end_table(self.start_);
        flatbuffers::WIPOffset::new(o.value())
    }
}

pub enum CameraRotateOffset {}
#[derive(Copy, Clone, Debug, PartialEq)]

pub struct CameraRotate<'a> {
    pub _tab: flatbuffers::Table<'a>,
}

impl<'a> flatbuffers::Follow<'a> for CameraRotate<'a> {
    type Inner = CameraRotate<'a>;
    #[inline]
    fn follow(buf: &'a [u8], loc: usize) -> Self::Inner {
        Self {
            _tab: flatbuffers::Table { buf: buf, loc: loc },
        }
    }
}

impl<'a> CameraRotate<'a> {
    #[inline]
    pub fn init_from_table(table: flatbuffers::Table<'a>) -> Self {
        CameraRotate { _tab: table }
    }
    #[allow(unused_mut)]
    pub fn create<'bldr: 'args, 'args: 'mut_bldr, 'mut_bldr>(
        _fbb: &'mut_bldr mut flatbuffers::FlatBufferBuilder<'bldr>,
        args: &'args CameraRotateArgs,
    ) -> flatbuffers::WIPOffset<CameraRotate<'bldr>> {
        let mut builder = CameraRotateBuilder::new(_fbb);
        builder.add_radians(args.radians);
        builder.finish()
    }

    pub const VT_RADIANS: flatbuffers::VOffsetT = 4;

    #[inline]
    pub fn radians(&self) -> f64 {
        self._tab
            .get::<f64>(CameraRotate::VT_RADIANS, Some(0.0))
            .unwrap()
    }
}

pub struct CameraRotateArgs {
    pub radians: f64,
}
impl<'a> Default for CameraRotateArgs {
    #[inline]
    fn default() -> Self {
        CameraRotateArgs { radians: 0.0 }
    }
}
pub struct CameraRotateBuilder<'a: 'b, 'b> {
    fbb_: &'b mut flatbuffers::FlatBufferBuilder<'a>,
    start_: flatbuffers::WIPOffset<flatbuffers::TableUnfinishedWIPOffset>,
}
impl<'a: 'b, 'b> CameraRotateBuilder<'a, 'b> {
    #[inline]
    pub fn add_radians(&mut self, radians: f64) {
        self.fbb_
            .push_slot::<f64>(CameraRotate::VT_RADIANS, radians, 0.0);
    }
    #[inline]
    pub fn new(_fbb: &'b mut flatbuffers::FlatBufferBuilder<'a>) -> CameraRotateBuilder<'a, 'b> {
        let start = _fbb.start_table();
        CameraRotateBuilder {
            fbb_: _fbb,
            start_: start,
        }
    }
    #[inline]
    pub fn finish(self) -> flatbuffers::WIPOffset<CameraRotate<'a>> {
        let o = self.fbb_.end_table(self.start_);
        flatbuffers::WIPOffset::new(o.value())
    }
}

pub enum CreateEntityOffset {}
#[derive(Copy, Clone, Debug, PartialEq)]

pub struct CreateEntity<'a> {
    pub _tab: flatbuffers::Table<'a>,
}

impl<'a> flatbuffers::Follow<'a> for CreateEntity<'a> {
    type Inner = CreateEntity<'a>;
    #[inline]
    fn follow(buf: &'a [u8], loc: usize) -> Self::Inner {
        Self {
            _tab: flatbuffers::Table { buf: buf, loc: loc },
        }
    }
}

impl<'a> CreateEntity<'a> {
    #[inline]
    pub fn init_from_table(table: flatbuffers::Table<'a>) -> Self {
        CreateEntity { _tab: table }
    }
    #[allow(unused_mut)]
    pub fn create<'bldr: 'args, 'args: 'mut_bldr, 'mut_bldr>(
        _fbb: &'mut_bldr mut flatbuffers::FlatBufferBuilder<'bldr>,
        args: &'args CreateEntityArgs<'args>,
    ) -> flatbuffers::WIPOffset<CreateEntity<'bldr>> {
        let mut builder = CreateEntityBuilder::new(_fbb);
        builder.add_y_pos(args.y_pos);
        builder.add_x_pos(args.x_pos);
        if let Some(x) = args.type_ {
            builder.add_type_(x);
        }
        builder.finish()
    }

    pub const VT_TYPE_: flatbuffers::VOffsetT = 4;
    pub const VT_X_POS: flatbuffers::VOffsetT = 6;
    pub const VT_Y_POS: flatbuffers::VOffsetT = 8;

    #[inline]
    pub fn type_(&self) -> Option<&'a str> {
        self._tab
            .get::<flatbuffers::ForwardsUOffset<&str>>(CreateEntity::VT_TYPE_, None)
    }
    #[inline]
    pub fn x_pos(&self) -> u32 {
        self._tab
            .get::<u32>(CreateEntity::VT_X_POS, Some(0))
            .unwrap()
    }
    #[inline]
    pub fn y_pos(&self) -> u32 {
        self._tab
            .get::<u32>(CreateEntity::VT_Y_POS, Some(0))
            .unwrap()
    }
}

pub struct CreateEntityArgs<'a> {
    pub type_: Option<flatbuffers::WIPOffset<&'a str>>,
    pub x_pos: u32,
    pub y_pos: u32,
}
impl<'a> Default for CreateEntityArgs<'a> {
    #[inline]
    fn default() -> Self {
        CreateEntityArgs {
            type_: None,
            x_pos: 0,
            y_pos: 0,
        }
    }
}
pub struct CreateEntityBuilder<'a: 'b, 'b> {
    fbb_: &'b mut flatbuffers::FlatBufferBuilder<'a>,
    start_: flatbuffers::WIPOffset<flatbuffers::TableUnfinishedWIPOffset>,
}
impl<'a: 'b, 'b> CreateEntityBuilder<'a, 'b> {
    #[inline]
    pub fn add_type_(&mut self, type_: flatbuffers::WIPOffset<&'b str>) {
        self.fbb_
            .push_slot_always::<flatbuffers::WIPOffset<_>>(CreateEntity::VT_TYPE_, type_);
    }
    #[inline]
    pub fn add_x_pos(&mut self, x_pos: u32) {
        self.fbb_.push_slot::<u32>(CreateEntity::VT_X_POS, x_pos, 0);
    }
    #[inline]
    pub fn add_y_pos(&mut self, y_pos: u32) {
        self.fbb_.push_slot::<u32>(CreateEntity::VT_Y_POS, y_pos, 0);
    }
    #[inline]
    pub fn new(_fbb: &'b mut flatbuffers::FlatBufferBuilder<'a>) -> CreateEntityBuilder<'a, 'b> {
        let start = _fbb.start_table();
        CreateEntityBuilder {
            fbb_: _fbb,
            start_: start,
        }
    }
    #[inline]
    pub fn finish(self) -> flatbuffers::WIPOffset<CreateEntity<'a>> {
        let o = self.fbb_.end_table(self.start_);
        flatbuffers::WIPOffset::new(o.value())
    }
}
