use byteorder::ByteOrder;
use EndianMode::*;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum EndianMode {
    Little,
    Big,
}

impl EndianMode {
    pub fn native() -> Self {
        let mut buf = [0; 4];
        byteorder::NativeEndian::write_u32(&mut buf, 1);
        if buf[0] == 1 {
            Little
        } else {
            Big
        }
    }

    pub fn write_u32(self, buf: &mut [u8], n: u32) {
        match self {
            Little => byteorder::LE::write_u32(buf, n),
            Big => byteorder::BE::write_u32(buf, n),
        }
    }

    pub fn write_u16(self, buf: &mut [u8], n: u16) {
        match self {
            Little => byteorder::LE::write_u16(buf, n),
            Big => byteorder::BE::write_u16(buf, n),
        }
    }

    pub fn read_u32(self, data: &[u8]) -> u32 {
        match self {
            Little => byteorder::LE::read_u32(data),
            Big => byteorder::BE::read_u32(data),
        }
    }

    pub fn read_u16(self, data: &[u8]) -> u16 {
        match self {
            Little => byteorder::LE::read_u16(data),
            Big => byteorder::BE::read_u16(data),
        }
    }
}
