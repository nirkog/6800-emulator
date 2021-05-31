use crate::disassembler;
use crate::memory_manager;

const MAX_INSTRUCTION_LENGTH: u16 = 3;

#[derive(Clone, Copy)]
pub struct ProcessorState {
    pub accumulator_a: u8,
    pub accumulator_b: u8,
    pub index_register: u16,
    pub program_counter: u16,
    pub stack_pointer: u16,
    pub condition_code_register: u8
}

pub struct Processor<'a> {
    state: ProcessorState,
    memory_manager: Option<&'a mut memory_manager::MemoryManager>
}

#[derive(Clone, Copy)]
pub struct AccessDetails {
    address: Option<u16>,
    value: Option<u8>,
}

pub enum ConditionCodeFlag {
    Carry,
    Overflow,
    Zero,
    Negative,
    InterrupMask,
    HalfCarry
}

#[derive(PartialEq, Eq, Debug)]
pub enum EmulationError {
    NoMemoryManager,
    DisassemblyError(disassembler::DisassemblyError)
}

// Get a specific bit from a variable
#[macro_export]
macro_rules! get_bit {
    ($value: expr, $bit: expr) => {
        (($value & (1 << $bit)) > 0)
    };
}

// Get the most significant byte of a word (16 bits)
#[macro_export]
macro_rules! word_get_high_byte {
    ($value: expr) => {
        (($value >> 8) as u8)
    }
}

// Get the least significant byte of a word (16 bits)
#[macro_export]
macro_rules! word_get_low_byte {
    ($value: expr) => {
        (($value & 0xff) as u8)
    }
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

    pub fn get_condition_code_flag(&self, flag: ConditionCodeFlag) -> bool {
        let index: u8 = match flag {
            ConditionCodeFlag::Carry => ProcessorState::CARRY_INDEX,
            ConditionCodeFlag::Overflow => ProcessorState::OVERFLOW_INDEX,
            ConditionCodeFlag::Zero => ProcessorState::ZERO_INDEX,
            ConditionCodeFlag::Negative => ProcessorState::NEGATIVE_INDEX,
            ConditionCodeFlag::InterrupMask => ProcessorState::INTERRUPT_MASK_INDEX,
            ConditionCodeFlag::HalfCarry => ProcessorState::HALF_CARRY_INDEX
        };

        get_bit!(self.condition_code_register, index)
    }
}

impl<'a> Processor<'a> {
    pub fn new() -> Processor<'a> {
        Processor {
            state: ProcessorState::new_empty(),
            memory_manager: None
        }
    }

    pub fn set_memory_manager(&mut self, memory_manager: &'a mut memory_manager::MemoryManager) {
        self.memory_manager = Some(memory_manager);
    }

    pub fn set_program_counter(&mut self, program_counter: u16) {
        self.state.program_counter = program_counter;
    }

    pub fn get_state(&self) -> ProcessorState {
        self.state
    }

    fn set_negative_flag(&mut self, result: u8) {
        self.state.set_condition_code_flag(ConditionCodeFlag::Negative, get_bit!(result, 7));
    }

    fn set_zero_flag(&mut self, result: u8) {
        self.state.set_condition_code_flag(ConditionCodeFlag::Zero, result == 0);
    }

    fn resolve_operand(&self, instruction_info: &disassembler::InstructionInfo, operand_index: usize) -> AccessDetails {
        let operands: &Vec<disassembler::OperandType> = instruction_info.operands.as_ref().unwrap();
        let mut access_details: AccessDetails = AccessDetails { address: None, value: None };
        let memory_manager = self.memory_manager.as_ref().unwrap();
        
        if instruction_info.opcode_info.addressing_mode == disassembler::AddressingMode::Direct {
            if let disassembler::OperandType::Immediate8(addr) = operands[operand_index] {
                access_details.address = Some(addr as u16);
                access_details.value = Some(memory_manager.read(addr as u16, 1)[0]);
            }
        } else if instruction_info.opcode_info.addressing_mode == disassembler::AddressingMode::Extended {
            if let disassembler::OperandType::Immediate16(addr) = operands[operand_index] {
                access_details.address = Some(addr);
                access_details.value = Some(memory_manager.read(addr, 1)[0]);
            }
        } else if instruction_info.opcode_info.addressing_mode == disassembler::AddressingMode::Immediate {
            if let disassembler::OperandType::Immediate8(value) = operands[operand_index] {
                access_details.value = Some(value);
            }
        } else if instruction_info.opcode_info.addressing_mode == disassembler::AddressingMode::Indexed {
            if let disassembler::OperandType::Immediate8(offset) = operands[operand_index] {
                access_details.address = Some(self.state.index_register + (offset as u16));
                access_details.value = Some(memory_manager.read(self.state.index_register + (offset as u16), 1)[0]);
            }
        } else if instruction_info.opcode_info.addressing_mode == disassembler::AddressingMode::Relative {
            if let disassembler::OperandType::Immediate8(offset) = operands[operand_index] {
                access_details.address = Some(self.state.program_counter + 2 + (offset as u16));
            }
        }

        access_details
    }

    fn get_accumulator_value(&self, accumulator: disassembler::OperandType) -> u8 {
        match accumulator {
            disassembler::OperandType::AccumulatorA => self.state.accumulator_a,
            disassembler::OperandType::AccumulatorB => self.state.accumulator_b,
            _ => 0
        }
    }

    fn set_accumulator_value(&mut self, accumulator: disassembler::OperandType, value: u8) {
        match accumulator {
            disassembler::OperandType::AccumulatorA => { self.state.accumulator_a = value },
            disassembler::OperandType::AccumulatorB => { self.state.accumulator_b = value },
            _ => {}
        };
    }

    fn write_to_memory(&mut self, address: u16, data: &[u8]) {
        self.memory_manager.as_mut().unwrap().write(address, data);
    }

    fn read_from_memory(&self, address: u16, size: u16) -> &[u8] {
        self.memory_manager.as_ref().unwrap().read(address, size)
    }

    fn set_addition_condition_codes(&mut self, accumulator: u8, operand: u8, result: u8) {
        let half_carry: bool;
        let overflow: bool;
        let carry: bool;
        let accumulator_msb: bool;
        let operand_msb: bool;
        let result_msb: bool;

        accumulator_msb = get_bit!(accumulator, 7);
        operand_msb = get_bit!(operand, 7);
        result_msb = get_bit!(result, 7);

        half_carry = (get_bit!(accumulator, 3) & get_bit!(operand, 3)) | (get_bit!(operand, 3) & !get_bit!(result, 3)) | (!get_bit!(result, 3) & get_bit!(accumulator, 3));
        overflow = (accumulator_msb & !result_msb & operand_msb) | (!accumulator_msb & !operand_msb & result_msb);
        carry = (accumulator_msb & operand_msb) | (operand_msb & !result_msb) | (!result_msb & accumulator_msb);
        self.set_negative_flag(result);
        self.set_zero_flag(result);
        self.state.set_condition_code_flag(ConditionCodeFlag::Overflow, overflow);
        self.state.set_condition_code_flag(ConditionCodeFlag::Carry, carry);
        self.state.set_condition_code_flag(ConditionCodeFlag::HalfCarry, half_carry);
    }

    fn set_subtraction_condition_codes(&mut self, accumulator: u8, operand: u8, result: u8) {
        let overflow: bool;
        let carry: bool;
        let accumulator_msb: bool;
        let operand_msb: bool;
        let result_msb: bool;

        accumulator_msb = get_bit!(accumulator, 7);
        operand_msb = get_bit!(operand, 7);
        result_msb = get_bit!(result, 7);

        overflow = (accumulator_msb & !result_msb & !operand_msb) | (!accumulator_msb & operand_msb & result_msb);
        carry = (!accumulator_msb & operand_msb) | (operand_msb & result_msb) | (result_msb & !accumulator_msb);
        self.set_negative_flag(result);
        self.set_zero_flag(result);
        self.state.set_condition_code_flag(ConditionCodeFlag::Overflow, overflow);
        self.state.set_condition_code_flag(ConditionCodeFlag::Carry, carry);
    }

    fn subtract_handler(&mut self, instruction_info: &disassembler::InstructionInfo) {
        let operands: &Vec<disassembler::OperandType> = instruction_info.operands.as_ref().unwrap();
        let operand: u8 = self.resolve_operand(instruction_info, 1).value.unwrap();
        let accumulator_value: u8;
        let result: u8;

        // Resolve the accumulator used
        accumulator_value = self.get_accumulator_value(operands[0]);

        // Calculate the result of the operation
        result = ((accumulator_value as i8) - (operand as i8)) as u8;
    
        // Set the conition codes
        self.set_subtraction_condition_codes(accumulator_value, operand, result);

        // Store the result in the used accumulator
        self.set_accumulator_value(operands[0], result);
    }

    fn add_b_to_a_handler(&mut self) {
        let a = self.state.accumulator_a;
        let b = self.state.accumulator_b;
        let result: u8; 

        // Calculate the addition result
        result = a + b;

        // Set the condition codes
        self.set_addition_condition_codes(a, b, result);

        // Store the result in accumulator A
        self.state.accumulator_a = result;
    }
    
    fn add_with_carry_handler(&mut self, instruction_info: &disassembler::InstructionInfo) {
        let operands: &Vec<disassembler::OperandType> = instruction_info.operands.as_ref().unwrap();
        let operand: u8 = self.resolve_operand(instruction_info, 1).value.unwrap();
        let accumulator_value: u8;
        let result: u8;

        // Resolve the accumulator used
        accumulator_value = self.get_accumulator_value(operands[0]);

        // Calculate the result of the operation
        result = accumulator_value + operand + (self.state.get_condition_code_flag(ConditionCodeFlag::Carry) as u8);

        // Set the condition codes
        self.set_addition_condition_codes(accumulator_value, operand, result);

        // Store the result in the used accumulator
        self.set_accumulator_value(operands[0], result);
    }

    fn add_handler(&mut self, instruction_info: &disassembler::InstructionInfo) {
        let operands: &Vec<disassembler::OperandType> = instruction_info.operands.as_ref().unwrap();
        let operand: u8 = self.resolve_operand(instruction_info, 1).value.unwrap();
        let accumulator_value: u8;
        let result: u8;

        // Resolve the accumulator used
        accumulator_value = self.get_accumulator_value(operands[0]);

        // Calculate the result of the operation
        result = accumulator_value + operand;

        // Set the condition codes
        self.set_addition_condition_codes(accumulator_value, operand, result);

        // Store the result in the used accumulator
        self.set_accumulator_value(operands[0], result);
    }

    fn and_handler(&mut self, instruction_info: &disassembler::InstructionInfo) {
        let operands: &Vec<disassembler::OperandType> = instruction_info.operands.as_ref().unwrap();
        let operand: u8 = self.resolve_operand(instruction_info, 1).value.unwrap();
        let accumulator_value: u8;
        let result: u8;

        // Resolve the accumulator used
        accumulator_value = self.get_accumulator_value(operands[0]);

        // Calculate the result of the operation
        result = accumulator_value & operand;

        // Set the condition codes
        self.set_negative_flag(result);
        self.set_zero_flag(result);
        self.state.set_condition_code_flag(ConditionCodeFlag::Overflow, false);

        // Store the result in the used accumulator
        self.set_accumulator_value(operands[0], result);
    }

    // TODO: Make a generic accumulaotr/memory arithmetic operation handler

    fn arithmetic_shift_left_handler(&mut self, instruction_info: &disassembler::InstructionInfo) {
        let operands: &Vec<disassembler::OperandType> = instruction_info.operands.as_ref().unwrap();
        let memory_access_details: AccessDetails;
        let accumulator_value: u8;
        let result: u8;
        let extra_bit: bool;

        if instruction_info.opcode_info.addressing_mode == disassembler::AddressingMode::Accumulator {
            accumulator_value = self.get_accumulator_value(operands[0]);

            result = accumulator_value << 1;
            extra_bit = get_bit!(accumulator_value, 7);

            self.set_accumulator_value(operands[0], result);
        } else {
            memory_access_details = self.resolve_operand(instruction_info, 0);

            result = memory_access_details.value.unwrap() << 1;
            extra_bit = get_bit!(memory_access_details.value.unwrap(), 7);

            self.write_to_memory(memory_access_details.address.unwrap(), &[result]);
        }

        self.set_negative_flag(result);
        self.set_zero_flag(result);
        self.state.set_condition_code_flag(ConditionCodeFlag::Carry, extra_bit);
        self.state.set_condition_code_flag(ConditionCodeFlag::Overflow, get_bit!(result, 7) ^ extra_bit);
    }

    fn arithmetic_shift_right_handler(&mut self, instruction_info: &disassembler::InstructionInfo) {
        let operands: &Vec<disassembler::OperandType> = instruction_info.operands.as_ref().unwrap();
        let memory_access_details: AccessDetails;
        let accumulator_value: u8;
        let result: u8;
        let first_bit: bool;

        if instruction_info.opcode_info.addressing_mode == disassembler::AddressingMode::Accumulator {
            accumulator_value = self.get_accumulator_value(operands[0]);

            result = (accumulator_value >> 1) | (accumulator_value & (1 << 7));
            first_bit = get_bit!(accumulator_value, 0);

            self.set_accumulator_value(operands[0], result);
        } else {
            memory_access_details = self.resolve_operand(instruction_info, 0);

            result = (memory_access_details.value.unwrap() << 1) | (memory_access_details.value.unwrap() & (1 << 7));
            first_bit = get_bit!(memory_access_details.value.unwrap(), 0);

            self.write_to_memory(memory_access_details.address.unwrap(), &[result]);
        }

        self.set_negative_flag(result);
        self.set_zero_flag(result);
        self.state.set_condition_code_flag(ConditionCodeFlag::Carry, first_bit);
        self.state.set_condition_code_flag(ConditionCodeFlag::Overflow, get_bit!(result, 7) ^ first_bit);
    }

    fn branch(&mut self, instruction_info: &disassembler::InstructionInfo) {
        let operand: u8 = self.resolve_operand(instruction_info, 0).value.unwrap();
        let mut signed_program_counter: i16 = self.state.program_counter as i16;
        
        signed_program_counter += operand as i8 as i16;
        signed_program_counter += 2;

        self.state.program_counter = signed_program_counter as u16;
    }

    fn branch_conditionally_handler<F>(&mut self, instruction_info: &disassembler::InstructionInfo, condition_evaluator: F) where 
        F: Fn(&ProcessorState) -> bool {
        if condition_evaluator(&self.state) {
            self.branch(instruction_info);
        }
    }

    fn bit_test_handler(&mut self, instruction_info: &disassembler::InstructionInfo) {
        let operands: &Vec<disassembler::OperandType> = instruction_info.operands.as_ref().unwrap();
        let operand: u8 = self.resolve_operand(instruction_info, 1).value.unwrap();
        let accumulator_value: u8 = self.get_accumulator_value(operands[0]);
        let result: u8;

        result = operand & accumulator_value;

        self.set_negative_flag(result);
        self.set_zero_flag(result);
        self.state.set_condition_code_flag(ConditionCodeFlag::Overflow, false);
    }

    fn branch_to_subroutine_handler(&mut self, instruction_info: &disassembler::InstructionInfo) {
        let program_counter_array: [u8; 2];

        // Save the address of the next instruction on the stack 
        self.state.program_counter += 2;
        program_counter_array = [word_get_high_byte!(self.state.program_counter), word_get_low_byte!(self.state.program_counter)];
        self.write_to_memory(self.state.stack_pointer, &program_counter_array);

        // Advance the stack after we pushed PC
        self.state.stack_pointer -= 2;

        // Branch to the subroutine address (subtract 2 from PC because the branch function adds it
        // and we added it too above)
        self.state.program_counter -= 2;
        self.branch(instruction_info);
    }

    fn compare(&mut self, operand1: u8, operand2: u8) {
        let result: u8 = operand1 - operand2;
        let operand1_msb: bool = get_bit!(operand1, 7);
        let operand2_msb: bool = get_bit!(operand2, 7);
        let result_msb: bool = get_bit!(result, 7);
        let overflow: bool;
        let carry: bool;

        overflow = (operand1_msb & !operand2_msb & !result_msb) | (!operand1_msb & operand2_msb & result_msb);
        carry = (!operand1_msb & operand2_msb) | (operand2_msb & result_msb) | (result_msb & !operand1_msb);

        self.set_negative_flag(result);
        self.set_zero_flag(result);
        self.state.set_condition_code_flag(ConditionCodeFlag::Carry, carry);
        self.state.set_condition_code_flag(ConditionCodeFlag::Overflow, overflow);
    }

    fn compare_accumulators_handler(&mut self) {
        self.compare(self.state.accumulator_a, self.state.accumulator_b);
    }

    fn clear_handler(&mut self, instruction_info: &disassembler::InstructionInfo) {
        let operands: &Vec<disassembler::OperandType> = instruction_info.operands.as_ref().unwrap();
        let memory_access_details: AccessDetails;

        if instruction_info.opcode_info.addressing_mode == disassembler::AddressingMode::Accumulator {
            self.set_accumulator_value(operands[0], 0);
        } else {
            memory_access_details = self.resolve_operand(instruction_info, 0);

            self.write_to_memory(memory_access_details.address.unwrap(), &[0]);
        }

        self.state.set_condition_code_flag(ConditionCodeFlag::Negative, false);
        self.state.set_condition_code_flag(ConditionCodeFlag::Zero, true);
        self.state.set_condition_code_flag(ConditionCodeFlag::Overflow, false);
        self.state.set_condition_code_flag(ConditionCodeFlag::Carry, false);
    }

    fn compare_handler(&mut self, instruction_info: &disassembler::InstructionInfo) {
        let operands: &Vec<disassembler::OperandType> = instruction_info.operands.as_ref().unwrap();
        let operand = self.resolve_operand(instruction_info, 1).value.unwrap();
        let accumulator_value = self.get_accumulator_value(operands[0]);

        self.compare(accumulator_value, operand);
    }

    fn complement_handler(&mut self, instruction_info: &disassembler::InstructionInfo) {
        let operands: &Vec<disassembler::OperandType> = instruction_info.operands.as_ref().unwrap();
        let memory_access_details: AccessDetails;
        let accumulator_value: u8;
        let result: u8;

        if instruction_info.opcode_info.addressing_mode == disassembler::AddressingMode::Accumulator {
            accumulator_value = self.get_accumulator_value(operands[0]);

            result = !accumulator_value;

            self.set_accumulator_value(operands[0], result);
        } else {
            memory_access_details = self.resolve_operand(instruction_info, 0);

            result = !memory_access_details.value.unwrap();

            self.write_to_memory(memory_access_details.address.unwrap(), &[result]);
        }

        self.set_negative_flag(result);
        self.set_zero_flag(result);
        self.state.set_condition_code_flag(ConditionCodeFlag::Carry, true);
        self.state.set_condition_code_flag(ConditionCodeFlag::Overflow, false);
    }

    fn compare_index_register_handler(&mut self, instruction_info: &disassembler::InstructionInfo) {
        let operand_address = self.resolve_operand(instruction_info, 0).address.unwrap();
        let operand_value = self.read_from_memory(operand_address, 2); 
        let index_register_low: u8 = (self.state.index_register & 0xFF) as u8;
        let index_register_high: u8 = (self.state.index_register >> 8) as u8;  
        let result_low: u8;
        let result_high: u8;
        let overflow: bool;

        result_low = index_register_low - operand_value[1]; 
        result_high = index_register_high - operand_value[0]; 

        overflow = (get_bit!(index_register_high, 7) & !get_bit!(operand_value[0], 7) & !get_bit!(result_high, 7)) |
                    (!get_bit!(index_register_high, 7) & get_bit!(operand_value[0], 7) & get_bit!(result_high, 7));

        self.set_negative_flag(result_high);
        self.set_zero_flag(result_low | result_high); // I'm not sure this works :)
        self.state.set_condition_code_flag(ConditionCodeFlag::Overflow, overflow);
    }

    fn decrement_handler(&mut self, instruction_info: &disassembler::InstructionInfo) {
        let operands: &Vec<disassembler::OperandType> = instruction_info.operands.as_ref().unwrap();
        let memory_access_details: AccessDetails;
        let accumulator_value: u8;
        let result: u8;

        if instruction_info.opcode_info.addressing_mode == disassembler::AddressingMode::Accumulator {
            accumulator_value = self.get_accumulator_value(operands[0]);

            result = accumulator_value - 1;

            self.set_accumulator_value(operands[0], result);
        } else {
            memory_access_details = self.resolve_operand(instruction_info, 0);

            result = memory_access_details.value.unwrap() - 1;

            self.write_to_memory(memory_access_details.address.unwrap(), &[result]);
        }

        self.set_negative_flag(result);
        self.set_zero_flag(result);
        self.state.set_condition_code_flag(ConditionCodeFlag::Overflow, result == 0b1111111);
    }

    fn decrement_index_register_handler(&mut self) {
        self.state.index_register -= 1;

        self.set_zero_flag(((self.state.index_register >> 8) as u8) | ((self.state.index_register as u8) & 0xff));
    }

    fn xor_handler(&mut self, instruction_info: &disassembler::InstructionInfo) {
        let operands: &Vec<disassembler::OperandType> = instruction_info.operands.as_ref().unwrap();
        let operand: u8 = self.resolve_operand(instruction_info, 1).value.unwrap();
        let accumulator_value: u8;
        let result: u8;

        // Resolve the accumulator used
        accumulator_value = self.get_accumulator_value(operands[0]);

        // Calculate the result of the operation
        result = accumulator_value ^ operand;

        // Set the condition codes
        self.set_negative_flag(result);
        self.set_zero_flag(result);
        self.state.set_condition_code_flag(ConditionCodeFlag::Overflow, false);

        // Store the result in the used accumulator
        self.set_accumulator_value(operands[0], result);
    }

    fn increment_handler(&mut self, instruction_info: &disassembler::InstructionInfo) {
        let operands: &Vec<disassembler::OperandType> = instruction_info.operands.as_ref().unwrap();
        let memory_access_details: AccessDetails;
        let accumulator_value: u8;
        let result: u8;

        if instruction_info.opcode_info.addressing_mode == disassembler::AddressingMode::Accumulator {
            accumulator_value = self.get_accumulator_value(operands[0]);

            result = accumulator_value - 1;

            self.set_accumulator_value(operands[0], result);
        } else {
            memory_access_details = self.resolve_operand(instruction_info, 0);

            result = memory_access_details.value.unwrap() - 1;

            self.write_to_memory(memory_access_details.address.unwrap(), &[result]);
        }

        self.set_negative_flag(result);
        self.set_zero_flag(result);
        self.state.set_condition_code_flag(ConditionCodeFlag::Overflow, result == 0b1111111);
    }

    fn increment_stack_pointer_handler(&mut self) {
        self.state.stack_pointer += 1;
    }

    fn increment_index_register_handler(&mut self) {
        self.state.index_register += 1;

        self.set_zero_flag(((self.state.index_register >> 8) as u8) | ((self.state.index_register as u8) & 0xff));
    }

    fn jump_handler(&mut self, instruction_info: &disassembler::InstructionInfo) {
        // TODO: Verify the operand index is correct in all handlers, the location of this TODO is
        // very random :)
        let operand: u16 = self.resolve_operand(instruction_info, 0).address.unwrap();

        self.state.program_counter = operand;
    }

    fn jump_to_subroutine_handler(&mut self, instruction_info: &disassembler::InstructionInfo) {
        let operand: u16 = self.resolve_operand(instruction_info, 0).address.unwrap();

        self.state.program_counter += instruction_info.opcode_info.instruction_length as u16;

        self.push((self.state.program_counter & 0xff) as u8);
        self.push((self.state.program_counter >> 8) as u8);

        self.state.program_counter = operand;
    }

    fn load_accumulator_handler(&mut self, instruction_info: &disassembler::InstructionInfo) {
        let operand: u8 = self.resolve_operand(instruction_info, 1).value.unwrap();

        self.state.accumulator_a = operand;

        self.set_zero_flag(self.state.accumulator_a);
        self.set_negative_flag(self.state.accumulator_a);
        self.state.set_condition_code_flag(ConditionCodeFlag::Overflow, false);
    }

    fn load_stack_pointer_handler(&mut self, instruction_info: &disassembler::InstructionInfo) {
        let operand_address: u16 = self.resolve_operand(instruction_info, 0).address.unwrap();
        let high_byte: u8 = self.read_from_memory(operand_address, 1)[0]; // TODO: Optimize this
        let low_byte: u8 = self.read_from_memory(operand_address + 1, 1)[0];

        self.state.stack_pointer = ((high_byte as u16) << 8) | (low_byte as u16);

        self.set_negative_flag(high_byte);
        self.set_zero_flag(high_byte | low_byte);
        self.state.set_condition_code_flag(ConditionCodeFlag::Overflow, false);
    }

    fn load_index_register_handler(&mut self, instruction_info: &disassembler::InstructionInfo) {
        let operand_address: u16 = self.resolve_operand(instruction_info, 0).address.unwrap();
        let high_byte: u8 = self.read_from_memory(operand_address, 1)[0]; // TODO: Optimize this
        let low_byte: u8 = self.read_from_memory(operand_address + 1, 1)[0];

        self.state.index_register = ((high_byte as u16) << 8) | (low_byte as u16);

        self.set_negative_flag(high_byte);
        self.set_zero_flag(high_byte | low_byte);
        self.state.set_condition_code_flag(ConditionCodeFlag::Overflow, false);
    }

    fn push(&mut self, byte: u8) {
        self.write_to_memory(self.state.stack_pointer, &[byte]);
        self.state.stack_pointer -= 1;
    }

    fn push_handler(&mut self, instruction_info: &disassembler::InstructionInfo) {
        let accumulator_value: u8 = self.get_accumulator_value(instruction_info.operands.as_ref().unwrap()[0]); 

        self.push(accumulator_value);
    }

    fn pop(&mut self) -> u8 {
        let result: u8 = self.read_from_memory(self.state.stack_pointer, 1)[0];

        self.state.stack_pointer += 1;

        result
    }

    fn pop_handler(&mut self, instruction_info: &disassembler::InstructionInfo) {
        let value: u8 = self.pop();
        self.set_accumulator_value(instruction_info.operands.as_ref().unwrap()[0], value);
    }

    fn transfer_a_to_b_handler(&mut self) {
        self.state.accumulator_b = self.state.accumulator_a;

        self.set_negative_flag(self.state.accumulator_a);
        self.set_zero_flag(self.state.accumulator_a);
        self.state.set_condition_code_flag(ConditionCodeFlag::Overflow, false);
    }

    fn transfer_a_to_condition_codes_handler(&mut self) {
        self.state.condition_code_register = self.state.accumulator_a;
    }

    fn transfer_b_to_a_handler(&mut self) {
        self.state.accumulator_a = self.state.accumulator_b;

        self.set_negative_flag(self.state.accumulator_a);
        self.set_zero_flag(self.state.accumulator_a);
        self.state.set_condition_code_flag(ConditionCodeFlag::Overflow, false);
    }

    fn transfer_condition_codes_to_a_handler(&mut self) {
        self.state.accumulator_a = self.state.condition_code_register | 0b11000000;
    }

    fn transfer_stack_pointer_to_index_register_handler(&mut self) {
        self.state.index_register = self.state.stack_pointer + 1;
    }

    fn transfer_index_register_to_stack_pointer_handler(&mut self) {
        self.state.stack_pointer = self.state.index_register - 1;
    }

    fn logical_shift_right_handler(&mut self, instruction_info: &disassembler::InstructionInfo) {
        let operands: &Vec<disassembler::OperandType> = instruction_info.operands.as_ref().unwrap();
        let memory_access_details: AccessDetails;
        let accumulator_value: u8;
        let result: u8;
        let first_bit: bool;

        if instruction_info.opcode_info.addressing_mode == disassembler::AddressingMode::Accumulator {
            accumulator_value = self.get_accumulator_value(operands[0]);

            result = (accumulator_value >> 1) & 0b1111111;
            first_bit = get_bit!(accumulator_value, 0);

            self.set_accumulator_value(operands[0], result);
        } else {
            memory_access_details = self.resolve_operand(instruction_info, 0);

            result = (memory_access_details.value.unwrap() << 1)  & 0b1111111;
            first_bit = get_bit!(memory_access_details.value.unwrap(), 0);

            self.write_to_memory(memory_access_details.address.unwrap(), &[result]);
        }

        self.set_negative_flag(result);
        self.set_zero_flag(result);
        self.state.set_condition_code_flag(ConditionCodeFlag::Carry, first_bit);
        self.state.set_condition_code_flag(ConditionCodeFlag::Overflow, get_bit!(result, 7) ^ first_bit);
    }

    fn negate_handler(&mut self, instruction_info: &disassembler::InstructionInfo) {
        let operands: &Vec<disassembler::OperandType> = instruction_info.operands.as_ref().unwrap();
        let memory_access_details: AccessDetails;
        let accumulator_value: u8;
        let result: u8;

        if instruction_info.opcode_info.addressing_mode == disassembler::AddressingMode::Accumulator {
            accumulator_value = self.get_accumulator_value(operands[0]);

            result = (-(accumulator_value as i8)) as u8;

            self.set_accumulator_value(operands[0], result);
        } else {
            memory_access_details = self.resolve_operand(instruction_info, 0);

            result = (-(memory_access_details.value.unwrap() as i8)) as u8;

            self.write_to_memory(memory_access_details.address.unwrap(), &[result]);
        }

        self.set_negative_flag(result);
        self.set_zero_flag(result);
        self.state.set_condition_code_flag(ConditionCodeFlag::Carry, result != 0);
        self.state.set_condition_code_flag(ConditionCodeFlag::Overflow, result == 0x80);
    }

    fn or_handler(&mut self, instruction_info: &disassembler::InstructionInfo) {
        let operands: &Vec<disassembler::OperandType> = instruction_info.operands.as_ref().unwrap();
        let operand: u8 = self.resolve_operand(instruction_info, 1).value.unwrap();
        let accumulator_value: u8;
        let result: u8;

        // Resolve the accumulator used
        accumulator_value = self.get_accumulator_value(operands[0]);

        // Calculate the result of the operation
        result = accumulator_value | operand;

        // Set the condition codes
        self.set_negative_flag(result);
        self.set_zero_flag(result);
        self.state.set_condition_code_flag(ConditionCodeFlag::Overflow, false);

        // Store the result in the used accumulator
        self.set_accumulator_value(operands[0], result);
    }

    fn rotate_left_handler(&mut self, instruction_info: &disassembler::InstructionInfo) {
        let operands: &Vec<disassembler::OperandType> = instruction_info.operands.as_ref().unwrap();
        let memory_access_details: AccessDetails;
        let accumulator_value: u8;
        let result: u8;
        let last_bit: bool;

        if instruction_info.opcode_info.addressing_mode == disassembler::AddressingMode::Accumulator {
            accumulator_value = self.get_accumulator_value(operands[0]);

            last_bit = (accumulator_value >> 7) == 1;
            result = (accumulator_value << 1) | (self.state.get_condition_code_flag(ConditionCodeFlag::Carry) as u8);

            self.set_accumulator_value(operands[0], result);
        } else {
            memory_access_details = self.resolve_operand(instruction_info, 0);

            last_bit = (memory_access_details.value.unwrap() >> 7) == 1;
            result = (memory_access_details.value.unwrap() << 1) | (self.state.get_condition_code_flag(ConditionCodeFlag::Carry) as u8);

            self.write_to_memory(memory_access_details.address.unwrap(), &[result]);
        }

        self.set_negative_flag(result);
        self.set_zero_flag(result);
        self.state.set_condition_code_flag(ConditionCodeFlag::Carry, last_bit);
        self.state.set_condition_code_flag(ConditionCodeFlag::Overflow, self.state.get_condition_code_flag(ConditionCodeFlag::Carry) ^ self.state.get_condition_code_flag(ConditionCodeFlag::Negative));
    }

    fn rotate_right_handler(&mut self, instruction_info: &disassembler::InstructionInfo) {
        let operands: &Vec<disassembler::OperandType> = instruction_info.operands.as_ref().unwrap();
        let memory_access_details: AccessDetails;
        let accumulator_value: u8;
        let result: u8;
        let first_bit: bool;

        if instruction_info.opcode_info.addressing_mode == disassembler::AddressingMode::Accumulator {
            accumulator_value = self.get_accumulator_value(operands[0]);

            first_bit = (accumulator_value & 1) == 1;
            result = (accumulator_value >> 1) | ((self.state.get_condition_code_flag(ConditionCodeFlag::Carry) as u8) << 7);

            self.set_accumulator_value(operands[0], result);
        } else {
            memory_access_details = self.resolve_operand(instruction_info, 0);

            first_bit = (memory_access_details.value.unwrap() & 1) == 1;
            result = (memory_access_details.value.unwrap() >> 1) | ((self.state.get_condition_code_flag(ConditionCodeFlag::Carry) as u8) << 7);

            self.write_to_memory(memory_access_details.address.unwrap(), &[result]);
        }

        self.set_negative_flag(result);
        self.set_zero_flag(result);
        self.state.set_condition_code_flag(ConditionCodeFlag::Carry, first_bit);
        self.state.set_condition_code_flag(ConditionCodeFlag::Overflow, self.state.get_condition_code_flag(ConditionCodeFlag::Carry) ^ self.state.get_condition_code_flag(ConditionCodeFlag::Negative));
    }

    fn return_handler(&mut self) {
        let program_counter_low: u8;
        let program_counter_high: u8;

        program_counter_high = self.pop();
        program_counter_low = self.pop();

        self.state.program_counter = ((program_counter_high as u16) << 8) | (program_counter_low as u16);
    }

    fn subtract_accumulators_handler(&mut self) {
        let result: u8;

        result = ((self.state.accumulator_a as i8) - (self.state.accumulator_b as i8)) as u8;

        self.set_subtraction_condition_codes(self.state.accumulator_a, self.state.accumulator_b, result);

        self.state.accumulator_a = result;
    }

    fn subtract_with_carry_handler(&mut self, instruction_info: &disassembler::InstructionInfo) {
        let operands: &Vec<disassembler::OperandType> = instruction_info.operands.as_ref().unwrap();
        let operand: u8 = self.resolve_operand(instruction_info, 1).value.unwrap();
        let accumulator_value: u8;
        let result: u8;

        // Resolve the accumulator used
        accumulator_value = self.get_accumulator_value(operands[0]);

        // Calculate the result of the operation
        result = ((accumulator_value as i8) - (operand as i8) - (self.state.get_condition_code_flag(ConditionCodeFlag::Carry) as i8)) as u8;
    
        // Set the conition codes
        self.set_subtraction_condition_codes(accumulator_value, operand, result);

        // Store the result in the used accumulator
        self.set_accumulator_value(operands[0], result);
    }

    fn store_accumulator_handler(&mut self, instruction_info: &disassembler::InstructionInfo) {
        let operands: &Vec<disassembler::OperandType> = instruction_info.operands.as_ref().unwrap();
        let operand_address: u16 = self.resolve_operand(instruction_info, 1).address.unwrap();
        let accumulator_value: u8 = self.get_accumulator_value(operands[0]);

        self.write_to_memory(operand_address, &[accumulator_value]);

        self.set_negative_flag(accumulator_value);
        self.set_zero_flag(accumulator_value);
        self.state.set_condition_code_flag(ConditionCodeFlag::Overflow, false);
    }

    fn store_stack_pointer_handler(&mut self, instruction_info: &disassembler::InstructionInfo) {
        let operand_address: u16 = self.resolve_operand(instruction_info, 1).address.unwrap();
        let high_byte: u8 = (self.state.stack_pointer >> 8) as u8;
        let low_byte: u8 = (self.state.stack_pointer & 0xff) as u8;

        self.write_to_memory(operand_address, &[high_byte, low_byte]);

        self.set_negative_flag(high_byte);
        self.set_zero_flag(high_byte | low_byte);
        self.state.set_condition_code_flag(ConditionCodeFlag::Overflow, false);
    }

    fn store_index_register_handler(&mut self, instruction_info: &disassembler::InstructionInfo) {
        let operand_address: u16 = self.resolve_operand(instruction_info, 1).address.unwrap();
        let high_byte: u8 = (self.state.index_register >> 8) as u8;
        let low_byte: u8 = (self.state.index_register & 0xff) as u8;

        self.write_to_memory(operand_address, &[high_byte, low_byte]);

        self.set_negative_flag(high_byte);
        self.set_zero_flag(high_byte | low_byte);
        self.state.set_condition_code_flag(ConditionCodeFlag::Overflow, false);
    }

    fn test_handler(&mut self, instruction_info: &disassembler::InstructionInfo) {
        let operands: &Vec<disassembler::OperandType> = instruction_info.operands.as_ref().unwrap();
        let memory_access_details: AccessDetails;
        let accumulator_value: u8;
        let result: u8;

        if instruction_info.opcode_info.addressing_mode == disassembler::AddressingMode::Accumulator {
            accumulator_value = self.get_accumulator_value(operands[0]);

            result = accumulator_value;

            self.set_accumulator_value(operands[0], result);
        } else {
            memory_access_details = self.resolve_operand(instruction_info, 0);

            result = memory_access_details.value.unwrap();

            self.write_to_memory(memory_access_details.address.unwrap(), &[result]);
        }

        self.set_negative_flag(result);
        self.set_zero_flag(result);
        self.state.set_condition_code_flag(ConditionCodeFlag::Carry, false);
        self.state.set_condition_code_flag(ConditionCodeFlag::Overflow, false);
    }

    fn decimal_adjust_a_handler(&mut self) {
        // TODO: Implement this
    }

    pub fn emulate_instruction(&mut self) -> Result<disassembler::InstructionInfo, EmulationError> {
        let instruction_info: disassembler::InstructionInfo;
        let memory_manager: &mut memory_manager::MemoryManager;
        let data_stream: &[u8]; 

        // Return an error if there is no memory manager defined
        if self.memory_manager.is_none() {
            return Err(EmulationError::NoMemoryManager);
        }
    
        memory_manager = self.memory_manager.as_mut().unwrap();

        // TODO: This has a bug. If we read from the end of the memory and the length of the final
        //       instruction is less the MAX_INSTRUCTION_SIZE, we will would cause an error in the
        //       memory manager
        data_stream = memory_manager.read(self.state.program_counter, MAX_INSTRUCTION_LENGTH);
        instruction_info = match disassembler::disassemble_instruction(data_stream) {
            Ok(info) => info,
            Err(err) => return Err(EmulationError::DisassemblyError(err))
        };

        // Emulate the instruction based on the opcode group
        match instruction_info.opcode_info.opcode {
            disassembler::Opcode::AddBToA => self.add_b_to_a_handler(),
            disassembler::Opcode::AddWithCarry => self.add_with_carry_handler(&instruction_info),
            disassembler::Opcode::Add => self.add_handler(&instruction_info),
            disassembler::Opcode::And => self.and_handler(&instruction_info),
            disassembler::Opcode::ArithmeticShiftLeft => self.arithmetic_shift_left_handler(&instruction_info),
            disassembler::Opcode::ArithmeticShiftRight => self.arithmetic_shift_right_handler(&instruction_info),
            disassembler::Opcode::BranchIfCarryClear => self.branch_conditionally_handler(&instruction_info, |state| !state.get_condition_code_flag(ConditionCodeFlag::Carry)),
            disassembler::Opcode::BranchIfCarrySet => self.branch_conditionally_handler(&instruction_info, |state| state.get_condition_code_flag(ConditionCodeFlag::Carry)),
            disassembler::Opcode::BranchIfEqual => self.branch_conditionally_handler(&instruction_info, |state| state.get_condition_code_flag(ConditionCodeFlag::Zero)),
            disassembler::Opcode::BranchIfGreaterThanEqual => self.branch_conditionally_handler(&instruction_info, |state| !(state.get_condition_code_flag(ConditionCodeFlag::Negative) ^ state.get_condition_code_flag(ConditionCodeFlag::Overflow))),
            disassembler::Opcode::BranchIfGreaterThan => self.branch_conditionally_handler(&instruction_info, |state| state.get_condition_code_flag(ConditionCodeFlag::Zero) & !(state.get_condition_code_flag(ConditionCodeFlag::Negative) ^ state.get_condition_code_flag(ConditionCodeFlag::Overflow))),
            disassembler::Opcode::BranchIfHigherThan => self.branch_conditionally_handler(&instruction_info, |state| !(state.get_condition_code_flag(ConditionCodeFlag::Carry) | state.get_condition_code_flag(ConditionCodeFlag::Zero))),
            disassembler::Opcode::BitTest => self.bit_test_handler(&instruction_info),
            disassembler::Opcode::BranchIfLessThanEqaul => self.branch_conditionally_handler(&instruction_info, |state| (state.get_condition_code_flag(ConditionCodeFlag::Zero) | (state.get_condition_code_flag(ConditionCodeFlag::Negative) ^ state.get_condition_code_flag(ConditionCodeFlag::Overflow)))),
            disassembler::Opcode::BranchIfLowerThanEqual => self.branch_conditionally_handler(&instruction_info, |state| state.get_condition_code_flag(ConditionCodeFlag::Carry) | state.get_condition_code_flag(ConditionCodeFlag::Zero)),
            disassembler::Opcode::BranchIfLessThan => self.branch_conditionally_handler(&instruction_info, |state| state.get_condition_code_flag(ConditionCodeFlag::Negative) ^ state.get_condition_code_flag(ConditionCodeFlag::Overflow)),
            disassembler::Opcode::BranchIfMinus => self.branch_conditionally_handler(&instruction_info, |state| state.get_condition_code_flag(ConditionCodeFlag::Negative)),
            disassembler::Opcode::BranchIfNotEqual => self.branch_conditionally_handler(&instruction_info, |state| !state.get_condition_code_flag(ConditionCodeFlag::Zero)),
            disassembler::Opcode::BranchIfPlus => self.branch_conditionally_handler(&instruction_info, |state| !state.get_condition_code_flag(ConditionCodeFlag::Negative)),
            disassembler::Opcode::BranchUnconditional => self.branch(&instruction_info),
            disassembler::Opcode::BranchToSubroutine => self.branch_to_subroutine_handler(&instruction_info),
            disassembler::Opcode::BranchIfOverflowClear => self.branch_conditionally_handler(&instruction_info, |state| !state.get_condition_code_flag(ConditionCodeFlag::Overflow)),
            disassembler::Opcode::BranchIfOverflowSet => self.branch_conditionally_handler(&instruction_info, |state| state.get_condition_code_flag(ConditionCodeFlag::Overflow)),
            disassembler::Opcode::CompareAAndB => self.compare_accumulators_handler(),
            disassembler::Opcode::ClearCarryFlag => self.state.set_condition_code_flag(ConditionCodeFlag::Carry, false),
            disassembler::Opcode::ClearInterruptMask => self.state.set_condition_code_flag(ConditionCodeFlag::InterrupMask, false),
            disassembler::Opcode::Clear=> self.clear_handler(&instruction_info),
            disassembler::Opcode::ClearOverflowFlag => self.state.set_condition_code_flag(ConditionCodeFlag::Overflow, false),
            disassembler::Opcode::Compare => self.compare_handler(&instruction_info),
            disassembler::Opcode::Complement => self.complement_handler(&instruction_info),
            disassembler::Opcode::CompareIndexRegister => self.compare_index_register_handler(&instruction_info),
            disassembler::Opcode::DecimalAdjustA => self.decimal_adjust_a_handler(),
            disassembler::Opcode::Decrement => self.decrement_handler(&instruction_info),
            disassembler::Opcode::DecrementStackPointer => { self.state.stack_pointer -= 1 },
            disassembler::Opcode::DecrementIndexRegister => self.decrement_index_register_handler(),
            disassembler::Opcode::Xor => self.xor_handler(&instruction_info),
            disassembler::Opcode::Increment => self.increment_handler(&instruction_info),
            disassembler::Opcode::IncrementStackPointer => self.increment_stack_pointer_handler(),
            disassembler::Opcode::IncrementIndexRegister => self.increment_index_register_handler(),
            disassembler::Opcode::Jump => self.jump_handler(&instruction_info),
            disassembler::Opcode::JumpToSubroutine => self.jump_to_subroutine_handler(&instruction_info),
            disassembler::Opcode::LoadAccumulator => self.load_accumulator_handler(&instruction_info),
            disassembler::Opcode::LoadStackPointer => self.load_stack_pointer_handler(&instruction_info),
            disassembler::Opcode::LoadIndexRegister => self.load_index_register_handler(&instruction_info),
            disassembler::Opcode::LogicalShiftRight => self.logical_shift_right_handler(&instruction_info),
            disassembler::Opcode::Negate => self.negate_handler(&instruction_info),
            disassembler::Opcode::Nop => {},
            disassembler::Opcode::Or => self.or_handler(&instruction_info),
            disassembler::Opcode::Push => self.push_handler(&instruction_info),
            disassembler::Opcode::Pop => self.pop_handler(&instruction_info),
            disassembler::Opcode::RotateLeft => self.rotate_left_handler(&instruction_info),
            disassembler::Opcode::RotateRight => self.rotate_right_handler(&instruction_info),
            // TODO: Return from interrupt
            disassembler::Opcode::Return => self.return_handler(),
            disassembler::Opcode::SubtractBFromA => self.subtract_accumulators_handler(),
            disassembler::Opcode::SubtractWithCarry => self.subtract_with_carry_handler(&instruction_info),
            disassembler::Opcode::SetCarryFlag => self.state.set_condition_code_flag(ConditionCodeFlag::Carry, true),
            disassembler::Opcode::SetInterruptMask => self.state.set_condition_code_flag(ConditionCodeFlag::InterrupMask, true),
            disassembler::Opcode::SetOverflowFlag => self.state.set_condition_code_flag(ConditionCodeFlag::Overflow, true),
            disassembler::Opcode::StoreAccumulator => self.store_accumulator_handler(&instruction_info),
            disassembler::Opcode::StoreStackPointer => self.store_stack_pointer_handler(&instruction_info),
            disassembler::Opcode::StoreIndexRegister => self.store_index_register_handler(&instruction_info),
            disassembler::Opcode::Subtract => self.subtract_handler(&instruction_info),
            // TODO: Software interrupt
            disassembler::Opcode::TransferAToB => self.transfer_a_to_b_handler(),
            disassembler::Opcode::TransferAToConditionCodes => self.transfer_a_to_condition_codes_handler(),
            disassembler::Opcode::TransferBToA => self.transfer_b_to_a_handler(),
            disassembler::Opcode::TransferConditionCodesToA => self.transfer_condition_codes_to_a_handler(),
            disassembler::Opcode::Test => self.test_handler(&instruction_info),
            disassembler::Opcode::TransferStackPointerToIndexRegister => self.transfer_stack_pointer_to_index_register_handler(),
            disassembler::Opcode::TransferIndexRegisterToStackPointer => self.transfer_index_register_to_stack_pointer_handler(),
            // TODO: Wait for interrupt
            _ => {}
        };

        Ok(instruction_info)
    }
}
