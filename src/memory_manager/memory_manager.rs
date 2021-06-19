// The size of the memory manager buffer (2 ^ 16 because of the 16 bit address range)
const BUFFER_SIZE: usize = 65536;

#[derive(Clone, Copy)]
pub struct MemoryManager {
    // TODO: Make the size of the buffer variable
    buffer: [u8; BUFFER_SIZE]
}

impl MemoryManager {
    pub fn new() -> MemoryManager {
        let instance = MemoryManager {
            buffer: [0; BUFFER_SIZE]
        };

        instance
    }
    
    pub fn read(&self, address: u16, size: u16) -> &[u8] {
        let start = address as usize;
        let end = (address + size) as usize;

        &self.buffer[start..end]
    }

    pub fn write(&mut self, address: u16, data: &[u8]) {
        let mut current_address = address as usize;

        for byte in data {
            self.buffer[current_address] = *byte;
            current_address += 1;
        }
    }
}
