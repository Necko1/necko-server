use std::io::{Error, ErrorKind, Write};
use bytes::{Buf};

pub type VarIntType = i32;

#[derive(Debug)]
pub struct VarInt(pub VarIntType);

impl VarInt {
    pub const MAX_SIZE: u8 = 5;

    pub const fn size(self) -> usize {
        match self.0 {
            0 => 1,
            n => (31 - n.leading_zeros() as usize) / 7 + 1,
        }
    }

    pub fn read(buffer: &mut &[u8]) -> Result<Self, Error> {
        let mut value: VarIntType = 0;
        
        for i in 0..Self::MAX_SIZE {
            if !buffer.has_remaining() {
                return Err(Error::new(ErrorKind::Other, "Buffer is corrupted"))
            }
            let byte = buffer.get_u8();
            value |= (VarIntType::from(byte) & 0x7F) << (7 * i);
            if byte & 0x80 == 0 {
                return Ok(VarInt(value))
            }
        }
        
        Err(Error::new(ErrorKind::InvalidData, "VarInt is too big"))
    }
    
    pub fn write(&mut self, mut buffer: impl Write) -> Result<(), Error> {
        let mut value = self.0 as u64;
        
        for _ in 0..Self::MAX_SIZE {
            let byte = (value & 0x7F) as u8;
            value >>= 7;
            if value == 0 {
                buffer.write_all(&[byte])?;
                break;
            }
            buffer.write_all(&[byte | 0x80])?;
        }
        
        Ok(())
    }
}