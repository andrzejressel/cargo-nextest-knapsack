fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
    #[test]
    fn crate_inline_test() {
        assert_eq!(1, 1);
    }
}