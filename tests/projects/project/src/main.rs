fn main() {
    println!("Hello, world!");
}

mod dir;

#[cfg(test)]
mod tests {
    #[test]
    fn root_inline_test() {
        assert_eq!(1, 1);
    }
}