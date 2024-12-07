use std::io::{Error, ErrorKind};
use bytes::{Buf, BufMut, Bytes, BytesMut};
use crate::types::{VarInt, VarIntType};
use core::str;
use uuid::Uuid;

#[derive(Debug)]
pub struct PacketByteBuffer(BytesMut);

impl PacketByteBuffer {
    
    pub fn empty() -> Self {
        PacketByteBuffer(BytesMut::new())
    }
    
    pub fn new(buffer: BytesMut) -> Self {
        PacketByteBuffer(buffer)
    }

    pub fn read_var_int(&mut self) -> Result<VarInt, Error> {
        let mut value: VarIntType = 0;

        for i in 0..VarInt::MAX_SIZE {
            let byte = self.read_u8()?;
            value |= (VarIntType::from(byte & 0x7F)) << (7 * i);
            if byte & 0x80 == 0 {
                return Ok(VarInt(value))
            }
        }

        Err(Error::new(ErrorKind::InvalidData, "VarInt is too big"))
    }

    pub fn write_var_int(&mut self, value: &VarInt) {
        let mut value = value.0 as u64;

        for _ in 0..VarInt::MAX_SIZE {
            let mut byte = (value & 0x7F) as u8;
            value >>= 7;
            if value != 0 {
                byte |= 0x80
            }
            self.0.put_u8(byte);
            if value == 0 { break }

        }
    }

    pub fn read_string(&mut self) -> Result<String, Error> {
        self.read_string_limited(i16::MAX as i32)
    }

    pub fn read_string_limited(&mut self, max_size: i32) -> Result<String, Error> {
        let size = self.read_var_int()?.0;
        if size > max_size {
            return Err(Error::new(ErrorKind::InvalidData, "String is too long"))
        }

        let bytes = self.copy_to_bytes(size as usize)?;
        if bytes.len() as i32 > max_size {
            return Err(Error::new(ErrorKind::InvalidData, "String is too big"))
        }

        match str::from_utf8(&bytes) {
            Ok(string) => Ok(string.to_string()),
            Err(e) => Err(Error::new(ErrorKind::Other, e))
        }
    }

    pub fn write_string(&mut self, value: &str) {
        self.write_string_limited(value, i16::MAX as i32)
    }

    pub fn write_string_limited(&mut self, value: &str, max_size: i32) {
        if value.len() > max_size as usize {
            panic!("String is too long")
        }

        self.write_var_int(&VarInt(value.len() as VarIntType));
        self.0.put(value.as_bytes());
    }
    
    pub fn read_u8(&mut self) -> Result<u8, Error> {
        if self.0.has_remaining() {
            Ok(self.0.get_u8())
        } else { Err(Error::new(ErrorKind::Other, "No bytes left")) }
    }

    pub fn read_u16(&mut self) -> Result<u16, Error> {
        if self.0.has_remaining() {
            Ok(self.0.get_u16())
        } else { Err(Error::new(ErrorKind::Other, "No bytes left")) }
    }
    
    pub fn read_i64(&mut self) -> Result<i64, Error> {
        if self.0.has_remaining() {
            Ok(self.0.get_i64())
        } else { Err(Error::new(ErrorKind::Other, "No bytes left")) }
    }

    pub fn write_i64(&mut self, value: i64) {
        self.0.put_i64(value)
    }

    pub fn read_uuid(&mut self) -> Result<Uuid, Error> {
        let mut bytes: [u8; 16] = [0; 16];
        self.copy_to_slice(&mut bytes)?;
        Ok(Uuid::from_slice(&bytes).expect("Failed to parse UUID"))
    }

    pub fn copy_to_bytes(&mut self, len: usize) -> Result<Bytes, Error> {
        if self.0.len() >= len {
            Ok(self.0.copy_to_bytes(len))
        } else { Err(Error::new(ErrorKind::Other, "No bytes left")) }
    }
    
    pub fn copy_to_slice(&mut self, slice: &mut [u8]) -> Result<(), Error> {
        if self.0.remaining() >= slice.len() {
            self.0.copy_to_slice(slice);
            Ok(())
        } else { Err(Error::new(ErrorKind::Other, "No bytes left")) }
    }

    pub fn buffer(&mut self) -> &mut BytesMut {
        &mut self.0
    }

}