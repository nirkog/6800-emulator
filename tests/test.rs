#[cfg(test)]
mod tests {
    #[test]
    fn first_disassembler_test() {
        assert_eq!(3, momulator::disassembler::disassemble());
    }
}
