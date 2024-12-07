use crate::buffer::PacketByteBuffer;
use crate::packets::{ClientboundPacket, Packet};
use crate::types::VarIntType;

pub struct StatusResponseClientbound<'a> {
    pub json_response: &'a str
}

impl<'a> StatusResponseClientbound<'a> {
    pub fn new(json_response: &'a str) -> Self {
        Self { json_response }
    }
}

impl<'a> Packet for StatusResponseClientbound<'a> { const PACKET_ID: VarIntType = 0x00; }

impl<'a> ClientboundPacket for StatusResponseClientbound<'a> {
    fn write(&self, buffer: &mut PacketByteBuffer) {
        buffer.write_string(self.json_response);
    }
}
