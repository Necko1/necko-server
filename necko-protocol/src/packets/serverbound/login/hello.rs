use std::io::Error;
use uuid::Uuid;
use crate::buffer::PacketByteBuffer;
use crate::packets::{Packet, ServerboundPacket};
use crate::types::VarIntType;

pub struct HelloServerbound {
    name: String,
    uuid: Uuid,
}

impl Packet for HelloServerbound { const PACKET_ID: VarIntType = 0x00; }

impl ServerboundPacket for HelloServerbound {
    fn read(buffer: &mut PacketByteBuffer) -> Result<Self, Error> {
        Ok(HelloServerbound {
            name: buffer.read_string_limited(16)?,
            uuid: buffer.read_uuid()?,
        })
    }
}