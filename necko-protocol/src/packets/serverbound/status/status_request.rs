use std::io::Error;
use crate::buffer::PacketByteBuffer;
use crate::packets::{Packet, ServerboundPacket};
use crate::types::VarIntType;

pub struct StatusRequestServerbound;

impl Packet for StatusRequestServerbound { const PACKET_ID: VarIntType = 0x00; }

impl ServerboundPacket for StatusRequestServerbound {
    fn read(_: &mut PacketByteBuffer) -> Result<Self, Error> {
        Ok(StatusRequestServerbound)
    }
}