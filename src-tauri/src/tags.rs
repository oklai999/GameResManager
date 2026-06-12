pub fn normalize_tag_name(input: &str) -> String {
    input.trim().split_whitespace().collect::<Vec<_>>().join(" ").to_lowercase()
}

pub fn normalize_tag_color(input: &str) -> anyhow::Result<String> {
    let value = input.trim();
    anyhow::ensure!(
        value.len() == 7
            && value.starts_with('#')
            && value[1..].bytes().all(|byte| byte.is_ascii_hexdigit()),
        "tag color must use #RRGGBB format"
    );
    Ok(value.to_ascii_uppercase())
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

    #[test]
    fn normalizes_valid_tag_color() {
        assert_eq!(normalize_tag_color("#5b8def").unwrap(), "#5B8DEF");
    }

    #[test]
    fn rejects_invalid_tag_colors() {
        for value in ["5B8DEF", "#fff", "#5B8DEFAA", "blue", ""] {
            assert!(normalize_tag_color(value).is_err(), "{value} should fail");
        }
    }
}
