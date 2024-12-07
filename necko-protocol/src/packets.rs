use std::io::Error;
use crate::buffer::PacketByteBuffer;
use crate::types::{VarInt, VarIntType};

pub mod clientbound;
pub mod serverbound;


pub const MAX_PACKET_SIZE: i32 = 2097151;

pub trait Packet {
    const PACKET_ID: VarIntType;
}

pub trait ServerboundPacket: Packet + Sized {
    fn read(buffer: &mut PacketByteBuffer) -> Result<Self, Error>;
}

pub trait ClientboundPacket: Packet {
    fn write(&self, buffer: &mut PacketByteBuffer);
}



#[derive(Debug)]
pub struct UnsignedPacket {
    pub id: VarInt,
    pub data: PacketByteBuffer
}