use std::cell::Cell;

pub struct Memory {
    data: Vec<Cell<u8>>,
}

impl Memory {
    pub fn new() -> Self {
        let mut data = Vec::with_capacity(0xFFFF);

        for _ in 0..0xFFFF {
            data.push(Cell::new(0));
        }

        Memory {
            data: data,
        }
    }

    pub fn read(&self, address: u16) -> u8 {
        self.data[address as usize].get()
    }

    pub fn write(&self, address: u16, data: u8) {
        self.data[address as usize].set(data);
    }
}
