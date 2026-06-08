pub fn normalize_tag_name(input: &str) -> String {
    input.trim().split_whitespace().collect::<Vec<_>>().join(" ").to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_tag_name() {
        assert_eq!(normalize_tag_name("  Chinese   Style  "), "chinese style");
    }

    #[test]
    fn empty_tag_stays_empty() {
        assert_eq!(normalize_tag_name("   "), "");
    }
}
