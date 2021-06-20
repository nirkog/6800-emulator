#[cfg(test)]
mod processor_tests {
    use momulator::processor::*;
    use momulator::memory_manager::*;
    use momulator::disassembler::*;
    use std::fs;
    
    // TODO: Add B To A Test

    #[test]
    fn processor_test_add_with_carry() {
        let mut memory_manager: MemoryManager = MemoryManager::new();
        let results: [u8; 4] = [0xFF, 0x3, 0xBE, 0xEF];
        let program: [u8; 12] = [0x89, results[0], 0x99, 0x9, 0xB9, 0x0, 0xa, 0xA9, 0xb, results[1], results[2], results[3]];
        let mut current_program_counter;
        let mut processor = Processor::new();
        let mut emulation_result: Result<InstructionInfo, EmulationError>;
        let mut state: ProcessorState;
        let mut i = 0;

        // Load the program and set the memory manager
        memory_manager.write(0, &program);
        processor.set_memory_manager(&mut memory_manager);

        // Set the program counter at the start of the code
        processor.set_program_counter(0);

        while i < 4 {
            current_program_counter = processor.get_state().program_counter;
            //processor.reset_state();
            processor.set_program_counter(current_program_counter);

            // Emulate the program and get the resulting state of the processor
            emulation_result = processor.emulate_instruction();
            state = processor.get_state();

            // Assert the instruction was emulated as excepted
            assert_eq!(emulation_result.is_err(), false);
            //assert_eq!(state.accumulator_a, results[i]);

            i += 1;
        }
    }

    #[test]
    fn processor_test_subtract() {
        let mut memory_manager: MemoryManager = MemoryManager::new();
        let program: [u8; 2] = [0x80, 0x15];
        let mut processor = Processor::new();
        let emulation_result: Result<InstructionInfo, EmulationError>;
        let state: ProcessorState;

        // Load the program and set the memory manager
        memory_manager.write(0, &program);
        processor.set_memory_manager(&mut memory_manager);

        // Set the program counter at the start of the code
        processor.set_program_counter(0);

        // Emulate the program and get the resulting state of the processor
        emulation_result = processor.emulate_instruction();
        state = processor.get_state();

        assert_eq!(emulation_result.is_err(), false);

        assert_eq!(state.accumulator_a as i8, -0x15);
    }

    #[test]
    fn processor_test_test_program() {
        let mut memory_manager = MemoryManager::new();
        let mut processor = Processor::new();
        let mut emulation_result: Result<InstructionInfo, EmulationError>;
        let mut state: ProcessorState;
        let error_address = 0x345;
        let success_address = error_address + 3;
        let mut i = 0;

        memory_manager.write(0, fs::read("./tests/test.bin").unwrap().as_slice());
        processor.set_memory_manager(&mut memory_manager);

        while i < 2000 {
            emulation_result = processor.emulate_instruction();
            state = processor.get_state();

            if emulation_result.is_err() {
                if emulation_result.unwrap_err() == EmulationError::NoMemoryManager {
                    println!("No memory manager");
                } else {
                    println!("Disassembly error, PC: {:#x}", state.program_counter);
                }
                assert_eq!(true, false);
            }

            // if state.program_counter > 0x270 {
            //     state.print();
            // }

            if state.program_counter == success_address {
                break;
            }

            assert_ne!(state.program_counter, error_address);
            
            i += 1
        }
    }
}
