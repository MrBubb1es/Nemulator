pub struct Memory {
    data: [Cell<u8>; 0xFFFF],
}

impl Memory {
    pub fn read(&self, address: u16) -> u8 {
        self.data[address as usize].get()
    }

    pub fn write(&self, address: u16, data: u8) {
        self.data[address as usize].set(data);
    }
}
