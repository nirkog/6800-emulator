use crate::disassembler;
use crate::memory_manager;

const MAX_INSTRUCTION_LENGTH: u16 = 3;

#[derive(Clone, Copy)]
pub struct ProcessorState {
    accumulator_a: u8,
    accumulator_b: u8,
    index_register: u16,
    program_counter: u16,
    stack_pointer: u16,
    condition_code_register: u8
}

pub enum ConditionCodeFlag {
    Carry,
    Overflow,
    Zero,
    Negative,
    InterrupMask,
    HalfCarry
}

pub struct Processor<'a> {
    state: ProcessorState,
    memory_manager: Option<&'a memory_manager::MemoryManager>
}

#[derive(PartialEq, Eq, Debug)]
pub enum EmulationError {
    NoMemoryManager,
    DisassemblyError(disassembler::DisassemblyError)
}

impl ProcessorState {
    // Indexes of the condition code register bits
    const CARRY_INDEX: u8 = 0;
    const OVERFLOW_INDEX: u8 = 1;
    const ZERO_INDEX: u8 = 2;
    const NEGATIVE_INDEX: u8 = 3;
    const INTERRUPT_MASK_INDEX: u8 = 4;
    const HALF_CARRY_INDEX: u8 = 5;

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

    fn _set_condition_code_flag(&mut self, index: u8, enable: bool) {
        match enable {
            true => self.condition_code_register |= 1 << index,
            false => self.condition_code_register &= !(1 << index)
        }
    }

    pub fn set_condition_code_flag(&mut self, flag: ConditionCodeFlag, enable: bool) {
        match flag {
            ConditionCodeFlag::Carry => self._set_condition_code_flag(ProcessorState::CARRY_INDEX, enable),
            ConditionCodeFlag::Overflow => self._set_condition_code_flag(ProcessorState::OVERFLOW_INDEX, enable),
            ConditionCodeFlag::Zero => self._set_condition_code_flag(ProcessorState::ZERO_INDEX, enable),
            ConditionCodeFlag::Negative => self._set_condition_code_flag(ProcessorState::NEGATIVE_INDEX, enable),
            ConditionCodeFlag::InterrupMask => self._set_condition_code_flag(ProcessorState::INTERRUPT_MASK_INDEX, enable),
            ConditionCodeFlag::HalfCarry => self._set_condition_code_flag(ProcessorState::HALF_CARRY_INDEX, enable),
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

    fn set_negative_flag(&mut self, result: u8) {
        self.state.set_condition_code_flag(ConditionCodeFlag::Negative, result & (1 << 7) > 0);
    }

    fn set_zero_flag(&mut self, result: u8) {
        self.state.set_condition_code_flag(ConditionCodeFlag::Zero, result == 0);
    }

    fn emulate_subtraction(&mut self, instruction_info: &disassembler::InstructionInfo) {
        let opcode: disassembler::Opcode = instruction_info.opcode_info.opcode;
        let overflow: bool;
        let carry: bool;
        let mut operand1: u8 = 0;
        let mut operand2: u8 = 0;
        let mut result: u8 = 0;
        let mut target: Option<&mut u8> = None;

        if instruction_info.opcode_info.addressing_mode == disassembler::AddressingMode::Inherent {
            if opcode == disassembler::Opcode::SubtractBFromA {
                operand1 = self.state.accumulator_a;
                operand2 = self.state.accumulator_b;
                result = self.state.accumulator_a - self.state.accumulator_b;
                target = Some(&mut self.state.accumulator_a);
            } 
        } else if instruction_info.opcode_info.addressing_mode == disassembler::AddressingMode::Immediate {
            //if instruction_info.operands[0] == disassembler::OperandType::AccumulatorA {

            //}
        }

        // Write the result to the target
        if target.is_some() {
            *(target.unwrap()) = result;
        }

        // Set the condition code register flags
        overflow = (((operand1 & (1 << 7)) & !(result & (1 << 7)) & !(operand1 & (1 << 7))) | (!(operand1 & (1 << 7)) & (operand2 & (1 << 7)) & (result & (1 << 7)))) > 0;
        carry = ((!(operand1 & (1 << 7)) & (operand2 & (1 << 7))) | ((operand2 & (1 << 7)) & (result & (1 << 7))) | ((result & (1 << 7)) & (operand1 & (1 << 7)))) > 0;
        self.set_negative_flag(result);
        self.set_zero_flag(result);
        self.state.set_condition_code_flag(ConditionCodeFlag::Overflow, overflow);
        self.state.set_condition_code_flag(ConditionCodeFlag::Carry, carry);
    }

    pub fn emulate_instruction(&mut self) -> Result<disassembler::InstructionInfo, EmulationError> {
        let instruction_info: disassembler::InstructionInfo;
        let memory_manager: &memory_manager::MemoryManager;
        let data_stream: &[u8]; 

        // Return an error if there is no memory manager defined
        if self.memory_manager.is_none() {
            return Err(EmulationError::NoMemoryManager);
        }
    
        memory_manager = self.memory_manager.unwrap();

        // TODO: This has a bug. If we read from the end of the memory and the length of the final
        // instruction is less the MAX_INSTRUCTION_SIZE, we will would cause an error in the
        // memory manager
        // Disassemble the next instruction
        data_stream = memory_manager.read(self.state.program_counter, MAX_INSTRUCTION_LENGTH);
        instruction_info = match disassembler::disassemble_instruction(data_stream) {
            Ok(info) => info,
            Err(err) => return Err(EmulationError::DisassemblyError(err))
        };

        // Emulate the instruction based on the opcode group
        match instruction_info.opcode_info.group {
            disassembler::OpcodeGroup::Subtract => self.emulate_subtraction(&instruction_info),
            _ => {}
        };

        Ok(instruction_info)
    }
}
