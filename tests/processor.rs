#[cfg(test)]
mod processor_tests {
    use momulator::processor::*;
    use momulator::memory_manager::*;
    use momulator::disassembler::*;

    #[test]
    fn test_subtract_instruction() {
        let mut memory_manager: MemoryManager = MemoryManager::new();
        let program: [u8; 2] = [0x80, 0x15];
        let mut processor = Processor::new();
        let emulation_result: Result<InstructionInfo, EmulationError>;
        let state: ProcessorState;

        // Load the program and set the memory manager
        memory_manager.write(0, &program);
        processor.set_memory_manager(&memory_manager);

        // Set the program counter at the start of the code
        processor.set_program_counter(0);

        // Emulate the program and get the resulting state of the processor
        emulation_result = processor.emulate_instruction();
        state = processor.get_state();

        assert_eq!(emulation_result.is_err(), false);

        assert_eq!(state.accumulator_a as i8, -0x15);
    }
}
