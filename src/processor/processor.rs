use crate::disassembler;
use crate::memory_manager;

#[derive(Copy)]
struct ProcessorState {
    accumulator_a: u8,
    accumulator_b: u8,
    index_register: u16,
    program_counter: u16,
    stack_pointer: u16,
    condition_code_register: u8
}

pub struct Processor<'a> {
    state: ProcessorState,
    memory_manager: Option<&'a memory_manager::MemoryManager>
}

impl ProcessorState {
    pub fn new_empty() -> ProcessorState {
        ProcessorState {
            accumulator_a: 0,
            accumulator_b: 0,
            index_register: 0,
            program_counter: 0,
            stack_pointer: 0,
            condition_code_register: 0
        }
    }
}

impl<'a> Processor<'a> {
    pub fn new() -> Processor<'a> {
        Processor {
            state: ProcessorState::new_empty(),
            memory_manager: None
        }
    }

    pub fn set_memory_manager(&mut self, memory_manager: &'a memory_manager::MemoryManager) {
        self.memory_manager = Some(memory_manager);
    }

    pub fn set_program_counter(&mut self, program_counter: u16) {
        self.state.program_counter = program_counter;
    }

    pub fn get_state(&self) -> ProcessorState {
        self.state
    }

    pub fn emulate_instruction() {
        
    }
}
