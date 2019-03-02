use enum_primitive_derive::Primitive;
use num_traits::FromPrimitive;

pub use generated::*;

use crate::{Parse, ParseError, ParserState, Result, Stream, ReadResult};
use crate::demo::message::bspdecal::*;
use crate::demo::message::classinfo::*;
use crate::demo::message::gameevent::*;
use crate::demo::message::generated::*;
use crate::demo::message::packetentities::*;
use crate::demo::message::stringtable::*;
use crate::demo::message::tempentities::*;
use crate::demo::message::usermessage::*;
use crate::demo::message::voice::*;
use bitstream_reader::{BitRead, LittleEndian};

pub mod classinfo;
pub mod generated;
pub mod stringtable;
pub mod voice;
pub mod bspdecal;
pub mod usermessage;
pub mod gameevent;
pub mod packetentities;
pub mod tempentities;

#[derive(Primitive, Debug)]
pub enum MessageType {
    File = 2,
    NetTick = 3,
    StringCmd = 4,
    SetConVar = 5,
    SigOnState = 6,
    Print = 7,
    ServerInfo = 8,
    ClassInfo = 10,
    SetPause = 11,
    CreateStringTable = 12,
    UpdateStringTable = 13,
    VoiceInit = 14,
    VoiceData = 15,
    ParseSounds = 17,
    SetView = 18,
    FixAngle = 19,
    BspDecal = 21,
    UserMessage = 23,
    EntityMessage = 24,
    GameEvent = 25,
    PacketEntities = 26,
    TempEntities = 27,
    PreFetch = 28,
    Menu = 29,
    GameEventList = 30,
    GetCvarValue = 31,
    CmdKeyValues = 32,
}

impl Parse for MessageType {
    fn parse(stream: &mut Stream, _state: &ParserState) -> Result<Self> {
        let raw = stream.read_int(6)?;
        let prop_type: Option<MessageType> = MessageType::from_u8(raw);
        prop_type.ok_or(ParseError::InvalidMessageType(raw))
    }
}

#[derive(Debug)]
pub enum Message {
    File(FileMessage),
    NetTick(NetTickMessage),
    StringCmd(StringCmdMessage),
    SetConVar(SetConVarMessage),
    SigOnState(SigOnStateMessage),
    Print(PrintMessage),
    ServerInfo(ServerInfoMessage),
    ClassInfo(ClassInfoMessage),
    SetPause(SetPauseMessage),
    CreateStringTable(CreateStringTableMessage),
    UpdateStringTable(UpdateStringTableMessage),
    VoiceInit(VoiceInitMessage),
    VoiceData(VoiceDataMessage),
    ParseSounds(ParseSoundsMessage),
    SetView(SetViewMessage),
    FixAngle(FixAngleMessage),
    BspDecal(BSPDecalMessage),
    UserMessage(UserMessage),
    EntityMessage(EntityMessage),
    GameEvent(GameEventMessage),
    PacketEntities(PacketEntitiesMessage),
    TempEntities(TempEntitiesMessage),
    PreFetch(PreFetchMessage),
    Menu(MenuMessage),
    GameEventList(GameEventListMessage),
    GetCvarValue(GetCvarValueMessage),
    CmdKeyValues(CmdKeyValuesMessage),
}

impl Parse for Message {
    fn parse(stream: &mut Stream, state: &ParserState) -> Result<Self> {
        let message_type = MessageType::parse(stream, state)?;
        Ok(match message_type {
            MessageType::File => Message::File(FileMessage::parse(stream, state)?),
            MessageType::NetTick => Message::NetTick(NetTickMessage::parse(stream, state)?),
            MessageType::StringCmd => Message::StringCmd(StringCmdMessage::parse(stream, state)?),
            MessageType::SetConVar => Message::SetConVar(SetConVarMessage::parse(stream, state)?),
            MessageType::SigOnState => Message::SigOnState(SigOnStateMessage::parse(stream, state)?),
            MessageType::Print => Message::Print(PrintMessage::parse(stream, state)?),
            MessageType::ServerInfo => Message::ServerInfo(ServerInfoMessage::parse(stream, state)?),
            MessageType::ClassInfo => Message::ClassInfo(ClassInfoMessage::parse(stream, state)?),
            MessageType::SetPause => Message::SetPause(SetPauseMessage::parse(stream, state)?),
            MessageType::CreateStringTable => Message::CreateStringTable(CreateStringTableMessage::parse(stream, state)?),
            MessageType::UpdateStringTable => Message::UpdateStringTable(UpdateStringTableMessage::parse(stream, state)?),
            MessageType::VoiceInit => Message::VoiceInit(VoiceInitMessage::parse(stream, state)?),
            MessageType::VoiceData => Message::VoiceData(VoiceDataMessage::parse(stream, state)?),
            MessageType::ParseSounds => Message::ParseSounds(ParseSoundsMessage::parse(stream, state)?),
            MessageType::SetView => Message::SetView(SetViewMessage::parse(stream, state)?),
            MessageType::FixAngle => Message::FixAngle(FixAngleMessage::parse(stream, state)?),
            MessageType::BspDecal => Message::BspDecal(BSPDecalMessage::parse(stream, state)?),
            MessageType::UserMessage => Message::UserMessage(UserMessage::parse(stream, state)?),
            MessageType::EntityMessage => Message::EntityMessage(EntityMessage::parse(stream, state)?),
            MessageType::GameEvent => Message::GameEvent(GameEventMessage::parse(stream, state)?),
            MessageType::PacketEntities => Message::PacketEntities(PacketEntitiesMessage::parse(stream, state)?),
            MessageType::TempEntities => Message::TempEntities(TempEntitiesMessage::parse(stream, state)?),
            MessageType::PreFetch => Message::PreFetch(PreFetchMessage::parse(stream, state)?),
            MessageType::Menu => Message::Menu(MenuMessage::parse(stream, state)?),
            MessageType::GameEventList => Message::GameEventList(GameEventListMessage::parse(stream, state)?),
            MessageType::GetCvarValue => Message::GetCvarValue(GetCvarValueMessage::parse(stream, state)?),
            MessageType::CmdKeyValues => Message::CmdKeyValues(CmdKeyValuesMessage::parse(stream, state)?),
        })
    }
}