use std::cell::Cell;

/// General purpose memory struct. Stores a vec of cell<u8>'s to allow for reading
/// and writing from different sources without having multiple mutable references.  
pub struct Memory {
    data: Vec<Cell<u8>>,
}

impl Memory {
    /// Create new memory with designated size. Memory is always accessed with
    /// 16 bit addresses, so memory is limited to a maximum accessable size of
    /// 0x10000, though it is possible to create memory with a larger size,
    pub fn new(size: usize) -> Self {
        let mut data = Vec::with_capacity(size);

        for _ in 0..=size {
            data.push(Cell::new(0));
        }

        Memory { data: data }
    }

    /// Create new memory from an existing vector of bytes. The memory will
    /// have the same capacity as the size of the vector passed in.
    pub fn from_vec(vec: Vec<u8>) -> Self {
        let mut data: Vec<Cell<u8>> = Vec::new();

        for byte in vec.into_iter() {
            data.push(Cell::new(byte));
        }

        Memory { data: data }
    }

    /// Read a single byte from a given address in memory
    pub fn read(&self, address: u16) -> u8 {
        self.data[address as usize].get()
    }

    /// Write a single byte to memory at a given address
    pub fn write(&self, address: u16, data: u8) {
        self.data[address as usize].set(data);
    }

    /// Get the size of the memory (for debugging purposes)
    pub fn size(&self) -> usize {
        self.data.len()
    }
}
