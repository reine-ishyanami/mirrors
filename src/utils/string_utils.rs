pub fn uppercase_first_letter(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + c.as_str(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uppercase_first_letter() {
        assert_eq!("Hello", uppercase_first_letter("hello"));
        assert_eq!("", uppercase_first_letter(""));
        assert_eq!("A", uppercase_first_letter("a"));
        assert_eq!("Abc", uppercase_first_letter("abc"));
    }
}
