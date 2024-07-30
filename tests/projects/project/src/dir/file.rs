#[cfg(test)]
mod tests {
    #[test]
    fn test_in_subdirectory() {
        assert_eq!(1, 1)
    }

    // Tests matching of test names
    #[test]
    fn test_in_subdirectory_2() {
        assert_eq!(1, 1)
    }
}