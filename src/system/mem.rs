use std::cell::Cell;

pub struct Memory {
    data: Vec<Cell<u8>>,
}

impl Memory {
    pub fn new(size: usize) -> Self {
        let mut data = Vec::with_capacity(size);

        for _ in 0..=size {
            data.push(Cell::new(0));
        }

        Memory { data: data }
    }

    pub fn from_vec(vec: Vec<u8>) -> Self {
        let mut data: Vec<Cell<u8>> = Vec::new();

        for byte in vec.into_iter() {
            data.push(Cell::new(byte));
        }

        Memory { data: data }
    }

    pub fn read(&self, address: u16) -> u8 {
        self.data[address as usize].get()
    }

    pub fn write(&self, address: u16, data: u8) {
        self.data[address as usize].set(data);
    }
}
