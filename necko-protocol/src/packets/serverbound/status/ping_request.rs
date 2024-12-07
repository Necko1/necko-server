use std::io::Error;
use crate::buffer::PacketByteBuffer;
use crate::packets::{Packet, ServerboundPacket};
use crate::types::VarIntType;

pub struct PingRequestServerbound {
    pub timestamp: i64
}

impl Packet for PingRequestServerbound { const PACKET_ID: VarIntType = 0x01; }

impl ServerboundPacket for PingRequestServerbound {
    fn read(buffer: &mut PacketByteBuffer) -> Result<Self, Error> {
        Ok(PingRequestServerbound {
            timestamp: buffer.read_i64()?
        })
    }
}