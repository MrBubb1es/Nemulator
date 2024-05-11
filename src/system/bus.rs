use crate::system::mem::Memory;

pub struct Bus {
    memory: Memory,
}

impl Bus {
    pub fn read(&self, address: u16) -> u8 {
        match address {
            0x0000..=0xFFFF => self.memory.read(address),
            _ => panic!("Invalid bus read at {address}"),
        }
    }

    pub fn write(&self, address: u16, data: u8) {
        match address {
            0x0000..=0xFFFF => self.memory.write(address, data),
            _ => panic!("Invalid bus write at {address} with data {data}"),
        }
    }
}
