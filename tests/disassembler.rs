#[cfg(test)]
mod disassembler_tests {
    use momulator::disassembler::*;

    #[test]
    fn test_substract_immediate8_instruction() {
        let sub_a_immediate: [u8; 2] = [0x80, 0x5];  
        let sub_a_immediate_disassembly = disassemble_instruction(&sub_a_immediate);

        // Make sure there were no errors
        assert_eq!(sub_a_immediate_disassembly.is_err(), false);
        let sub_a_immediate_disassembly = sub_a_immediate_disassembly.unwrap();

        // Test the instruction was disassembled correctly
        assert_eq!(sub_a_immediate_disassembly.opcode_info.opcode, Opcode::SubstractAImmediate);
        assert_eq!(sub_a_immediate_disassembly.opcode_info.group, OpcodeGroup::Substract);
        assert_eq!(sub_a_immediate_disassembly.opcode_info.instruction_length, 2);
        assert_eq!(sub_a_immediate_disassembly.opcode_info.cycles, 2);
        assert_eq!(sub_a_immediate_disassembly.operands.len(), 2);
        assert_eq!(sub_a_immediate_disassembly.operands[0].operand_type, OperandType::AccumulatorA);
        assert_eq!(sub_a_immediate_disassembly.operands[1].operand_type, OperandType::Immediate8);
        assert_eq!(sub_a_immediate_disassembly.operands[1].operand_value, Some(0x5));
    }
}
