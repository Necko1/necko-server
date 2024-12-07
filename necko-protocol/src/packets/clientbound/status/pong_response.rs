use crate::buffer::PacketByteBuffer;
use crate::packets::{ClientboundPacket, Packet};
use crate::types::VarIntType;

pub struct PongResponseClientbound {
    pub timestamp: i64
}

impl PongResponseClientbound {
    pub fn new(timestamp: i64) -> Self {
        Self { timestamp }
    }
}

impl Packet for PongResponseClientbound { const PACKET_ID: VarIntType = 0x01; }

impl ClientboundPacket for PongResponseClientbound {
    fn write(&self, buffer: &mut PacketByteBuffer) {
        buffer.write_i64(self.timestamp);
    }
}