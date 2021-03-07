/// All 6800 opcodes
#[derive(Debug, PartialEq, Eq)]
pub enum Opcode {
    Nop,
    TransferAToConditionCodes,
    TransferConditionCodesToA,
    IncrementIndexRegister,
    DecrementIndexRegister,
    ClearOverflowFlag,
    SetOverflowFlag,
    ClearCarryFlag,
    SetCarryFlag,
    ClearInterruptMask,
    SetInterruptMask,
    SubstractAImmediate,
    SubstractADirect,
    SubstractAIndexed,
    SubstractAExtended,
}

/// All possible operand types
#[derive(Debug, PartialEq, Eq)]
pub enum OperandType {
    AccumulatorA,
    AccumulatorB,
    IndexRegister,
    Immediate8,
    Immediate16,
    ConditionCodeRegister
}

/// Opcode groups that group together opcodes with different codes but similar meaning
#[derive(Debug, PartialEq, Eq)]
pub enum OpcodeGroup {
    Substract,
    TransferRegisters,
    IncrementRegister,
    DecrementRegister,
    ClearFlag,
    SetFlag,
    NoGroup
}

/// Contains basic information about an opcode
pub struct OpcodeInfo {
    pub opcode: Opcode,
    pub group: OpcodeGroup,
    pub instruction_length: u8,
    pub cycles: u8
}

/// Contains the relevant information about an operand
pub struct OperandInfo {
    pub operand_type: OperandType,
    pub operand_value: Option<u16>
}

/// Containts the disassembled information about an instruction
pub struct InstructionInfo {
    pub opcode_info: OpcodeInfo,
    pub operands: Vec<OperandInfo>
}

/// Errors that can arise during disassembly
#[derive(Debug, PartialEq, Eq)]
pub enum DisassemblyError {
    InvalidOpcodeByte,
    MachineCodeTooShort
}

/// Match a byte to its disassembled opcode
///
/// # Errors
/// The function might return an InvalidOpcodeByte error if an invalid byte was given (
/// A byte that does not correspond to any opcode).
fn match_byte_to_opcode_info(byte: u8) -> Result<OpcodeInfo, DisassemblyError> {
    let opcode_info: OpcodeInfo = match byte {
        0x01 => OpcodeInfo { opcode: Opcode::Nop, group: OpcodeGroup::NoGroup, instruction_length: 1, cycles: 1 },
        0x06 => OpcodeInfo { opcode: Opcode::TransferAToConditionCodes, group: OpcodeGroup::TransferRegisters, instruction_length: 1, cycles: 2},
        0x07 => OpcodeInfo { opcode: Opcode::TransferConditionCodesToA, group: OpcodeGroup::TransferRegisters, instruction_length: 1, cycles: 2},
        0x08 => OpcodeInfo { opcode: Opcode::IncrementIndexRegister, group: OpcodeGroup::IncrementRegister, instruction_length: 1, cycles: 4},
        0x09 => OpcodeInfo { opcode: Opcode::DecrementIndexRegister, group: OpcodeGroup::DecrementRegister, instruction_length: 1, cycles: 4},
        0x0A => OpcodeInfo { opcode: Opcode::ClearOverflowFlag, group: OpcodeGroup::ClearFlag, instruction_length: 1, cycles: 2},
        0x0B => OpcodeInfo { opcode: Opcode::SetOverflowFlag, group: OpcodeGroup::SetFlag, instruction_length: 1, cycles: 2},
        0x0C => OpcodeInfo { opcode: Opcode::ClearCarryFlag, group: OpcodeGroup::ClearFlag, instruction_length: 1, cycles: 2},
        0x0D => OpcodeInfo { opcode: Opcode::SetCarryFlag, group: OpcodeGroup::SetFlag, instruction_length: 1, cycles: 2},
        0x0E => OpcodeInfo { opcode: Opcode::ClearInterruptMask, group: OpcodeGroup::ClearFlag, instruction_length: 1, cycles: 2},
        0x0F => OpcodeInfo { opcode: Opcode::SetInterruptMask, group: OpcodeGroup::SetFlag, instruction_length: 1, cycles: 2},
        0x80 => OpcodeInfo { opcode: Opcode::SubstractAImmediate, group: OpcodeGroup::Substract, instruction_length: 2, cycles: 2 },
        0x90 => OpcodeInfo { opcode: Opcode::SubstractADirect, group: OpcodeGroup::Substract, instruction_length: 2, cycles: 3 },
        0xA0 => OpcodeInfo { opcode: Opcode::SubstractAIndexed, group: OpcodeGroup::Substract, instruction_length: 2, cycles: 5 },
        0xB0 => OpcodeInfo { opcode: Opcode::SubstractAExtended, group: OpcodeGroup::Substract, instruction_length: 3, cycles: 4 },
        _ => return Err(DisassemblyError::InvalidOpcodeByte)
    };

    Ok(opcode_info)
}

fn disassemble_operands(opcode_info: &OpcodeInfo, data: &[u8]) -> Vec<OperandInfo> {
    let operands = match opcode_info.opcode {
        Opcode::Nop => vec![],
        Opcode::TransferAToConditionCodes => vec![OperandInfo { operand_type: OperandType::AccumulatorA, operand_value: None }, OperandInfo { operand_type: OperandType::ConditionCodeRegister, operand_value: None }],
        Opcode::TransferConditionCodesToA => vec![OperandInfo { operand_type: OperandType::ConditionCodeRegister, operand_value: None }, OperandInfo { operand_type: OperandType::AccumulatorA, operand_value: None }],
        Opcode::IncrementIndexRegister => vec![OperandInfo { operand_type: OperandType::IndexRegister, operand_value: None }],
        Opcode::DecrementIndexRegister => vec![OperandInfo { operand_type: OperandType::IndexRegister, operand_value: None }],
        Opcode::ClearOverflowFlag => vec![OperandInfo { operand_type: OperandType::ConditionCodeRegister, operand_value: None }],
        Opcode::SetOverflowFlag => vec![OperandInfo { operand_type: OperandType::ConditionCodeRegister, operand_value: None }],
        Opcode::ClearCarryFlag => vec![OperandInfo { operand_type: OperandType::ConditionCodeRegister, operand_value: None }],
        Opcode::SetCarryFlag => vec![OperandInfo { operand_type: OperandType::ConditionCodeRegister, operand_value: None }],
        Opcode::ClearInterruptMask => vec![OperandInfo { operand_type: OperandType::ConditionCodeRegister, operand_value: None }],
        Opcode::SetInterruptMask => vec![OperandInfo { operand_type: OperandType::ConditionCodeRegister, operand_value: None }],
        Opcode::SubstractAImmediate => vec![OperandInfo { operand_type: OperandType::AccumulatorA, operand_value: None }, OperandInfo { operand_type: OperandType::Immediate8, operand_value: Some(data[1] as u16) }],
        Opcode::SubstractADirect => vec![OperandInfo { operand_type: OperandType::AccumulatorA, operand_value: None }, OperandInfo { operand_type: OperandType::Immediate8, operand_value: Some(data[1] as u16) }],
        Opcode::SubstractAIndexed => vec![OperandInfo { operand_type: OperandType::AccumulatorA, operand_value: None }, OperandInfo { operand_type: OperandType::Immediate8, operand_value: Some(data[1] as u16) }, OperandInfo { operand_type: OperandType::IndexRegister, operand_value: None }],
        Opcode::SubstractAExtended => vec![OperandInfo { operand_type: OperandType::AccumulatorA, operand_value: None }, OperandInfo { operand_type: OperandType::Immediate16, operand_value: Some((((data[1] as u16) << 8) | (data[2] as u16)) as u16) }],
    };

    operands
}

/// Disassemble the next instruction in a byte stream
pub fn disassemble_instruction(data: &[u8]) -> Result<InstructionInfo, DisassemblyError> {
    let opcode_info = match data.len() {
        0 => return Err(DisassemblyError::MachineCodeTooShort),
        _ => match_byte_to_opcode_info(data[0])
    };
    
    let opcode_info = match opcode_info {
        Ok(info) => info,
        Err(err) => return Err(err)
    };

    // Return an error if data is too short for operands
    if data.len() < opcode_info.instruction_length as usize {
        return Err(DisassemblyError::MachineCodeTooShort);
    }

    let operands = disassemble_operands(&opcode_info, data); 

    Ok(InstructionInfo { opcode_info, operands })
}
