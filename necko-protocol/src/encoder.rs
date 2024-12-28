use std::io::{Error, ErrorKind, Write};
use bytes::{BufMut, BytesMut};
use crate::buffer::PacketByteBuffer;
use crate::packets::{ClientboundPacket, MAX_PACKET_SIZE};
use crate::types::VarInt;

#[derive(Debug)]
pub struct Encoder {
    buffer: BytesMut
}

impl Encoder {
    pub fn new() -> Self {
        Encoder {
            buffer: BytesMut::with_capacity(1024)
        }
    }

    pub fn append<P: ClientboundPacket>(&mut self, packet: &P) -> Result<(), Error> {
        let start_len = self.buffer.len();
        let mut writer = (&mut self.buffer).writer();

        let mut buffer = PacketByteBuffer::empty();
        VarInt(P::PACKET_ID)
            .write(&mut writer)?;
        packet.write(&mut buffer);

        writer.write(buffer.buffer())?;

        let packet_len = self.buffer.len() - start_len;

        if packet_len > MAX_PACKET_SIZE as usize {
            return Err(Error::new(ErrorKind::Other, "Invalid packet size"))
        }

        let packet_len_size = VarInt(packet_len as i32).size();

        self.buffer.put_bytes(0, packet_len_size);
        self.buffer.copy_within(
            start_len..start_len + packet_len,
            start_len + packet_len_size);

        let front = &mut self.buffer[start_len..];
        VarInt(packet_len as i32)
            .write(front)?;

        Ok(())
    }

    pub fn take(&mut self) -> BytesMut {
        self.buffer.split()
    }

}