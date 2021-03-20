#[cfg(test)]
mod memory_manager_tests {
    use momulator::memory_manager::*;

    #[test]
    fn test_read_write() {
        let mut memory_manager = MemoryManager::create();
        let data: [u8; 4] = [1, 2, 3, 4];
        let result: &[u8];
        let mut i = 0;

        memory_manager.write(0, &[1, 2, 3, 4]);
        result = memory_manager.read(0, 4); 

        while i < 4 {
            assert_eq!(data[i], result[i]);
            i += 1;
        }
    }
}
