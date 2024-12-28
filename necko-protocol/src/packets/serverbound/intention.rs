use std::io::Error;
use crate::buffer::PacketByteBuffer;
use crate::packets::{Packet, ServerboundPacket};
use crate::types::{VarInt, VarIntType};

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum NextState {
    None,
    Status,
    Login,
    Transfer
}

impl From<VarInt> for NextState {
    fn from(value: VarInt) -> Self {
        match value.0 { 
            1 => NextState::Status,
            2 => NextState::Login,
            3 => NextState::Transfer,
            _ => unimplemented!("VarInt value out of range")
        }
    }
}

pub struct IntentionServerbound {
    pub protocol_version: VarInt,
    pub server_address: String,   // limited to 255
    pub server_port: u16,
    pub next_state: NextState
}

impl Packet for IntentionServerbound { const PACKET_ID: VarIntType = 0x00; }

impl ServerboundPacket for IntentionServerbound {
    fn read(buffer: &mut PacketByteBuffer) -> Result<Self, Error> {
        Ok(IntentionServerbound {
            protocol_version: buffer.read_var_int()?,
            server_address: buffer.read_string_limited(255)?,
            server_port: buffer.read_u16()?,
            next_state: buffer.read_var_int()?.into()
        })
    }
}