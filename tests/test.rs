#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }

    #[test]
    fn add_test() {
        assert_eq!(momulator::add(3, 4), 7);
    }
}
