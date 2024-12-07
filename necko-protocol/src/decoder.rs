use std::io::{Error, ErrorKind};
use bytes::{Buf, BytesMut};
use crate::buffer::PacketByteBuffer;
use crate::packets::{UnsignedPacket, MAX_PACKET_SIZE};
use crate::types::VarInt;

#[derive(Debug)]
pub struct Decoder {
    buffer: BytesMut
}

impl Decoder {
    pub fn new() -> Self {
        Decoder {
            buffer: BytesMut::new()
        }
    }

    pub fn decode(&mut self) -> Result<Option<UnsignedPacket>, Error> {
        let mut buffer = &self.buffer[..];

        let packet_len = match VarInt::read(&mut buffer) {
            Ok(var_int) => var_int,
            Err(e) => {
                if e.kind() == ErrorKind::Other {
                    return Ok(None)
                }
                return Err(e)
            }
        };
        let packet_length = packet_len.0;

        if packet_length < 0 || packet_length > MAX_PACKET_SIZE {
            return Err(Error::new(ErrorKind::Other, "Invalid packet size"))
        }

        if buffer.len() < packet_length as usize {
            return Ok(None)
        }

        let packet_len_size = packet_len.size();

        self.buffer.advance(packet_len_size);
        let mut data = self.buffer.split_to(packet_length as usize);
        buffer = &data[..];

        let packet_id = VarInt::read(&mut buffer)?;

        data.advance(data.len() - buffer.len());
        Ok(Some(
            UnsignedPacket {
                id: packet_id, data: PacketByteBuffer::new(data),
            }
        ))
    }

    pub fn append_bytes(&mut self, bytes: BytesMut) {
        self.buffer.unsplit(bytes);
    }

    pub fn append_slice(&mut self, bytes: &[u8]) {
        self.buffer.extend_from_slice(bytes);
    }

    pub fn take_capacity(&mut self) -> BytesMut {
        self.buffer.split_off(self.buffer.len())
    }

    pub fn reserve(&mut self, additional: usize) {
        self.buffer.reserve(additional);
    }

}