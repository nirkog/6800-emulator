#[cfg(test)]
mod disassembler_tests {
    use momulator::disassembler::*;

    #[test]
    fn test_substract_immediate8_instruction() {
        let sub_a_immediate: [u8; 2] = [0x80, 0x5];  
        let sub_a_immediate_disassembly = disassemble_instruction(&sub_a_immediate);
        let sub_a_operands: Vec<OperandType>;

        // Make sure there were no errors
        assert_eq!(sub_a_immediate_disassembly.is_err(), false);
        let sub_a_immediate_disassembly = sub_a_immediate_disassembly.unwrap();

        // Test the instruction was disassembled correctly
        assert_eq!(sub_a_immediate_disassembly.opcode_info.opcode, Opcode::Subtract);
        assert_eq!(sub_a_immediate_disassembly.opcode_info.instruction_length, 2);
        assert_eq!(sub_a_immediate_disassembly.opcode_info.cycles, 2);
        assert_ne!(sub_a_immediate_disassembly.operands, None);
        sub_a_operands = sub_a_immediate_disassembly.operands.unwrap();
        assert_eq!(sub_a_operands.len(), 2);
        assert_eq!(sub_a_operands[0], OperandType::AccumulatorA);
        assert_eq!(sub_a_operands[1], OperandType::Immediate8(0x5));
    }

    #[test]
    fn test_bad_opcode() {
        let bad_opcode_value_data: [u8; 1] = [0x4];
        let disassembly = disassemble_instruction(&bad_opcode_value_data); 

        // Make sure disassembling and invalid opcode results in a disassembly error
        assert_eq!(disassembly.is_err(), true);

        let disassembly = disassembly.unwrap_err();
        assert_eq!(disassembly, DisassemblyError::InvalidOpcodeByte);
    }
}
