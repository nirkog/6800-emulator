pub struct MemoryManager {
    buffer: [u8; 2 ^ 16]
}

impl MemoryManager {
    pub fn create() -> MemoryManager {
        let instance = MemoryManager {
            buffer: [0; 2 ^ 16]
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
