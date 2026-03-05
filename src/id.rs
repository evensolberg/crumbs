use rand::RngExt;

const ALPHABET: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789";

pub fn generate(prefix: &str) -> String {
    let mut rng = rand::rng();
    let suffix: String = (0..3)
        .map(|_| ALPHABET[rng.random_range(0..ALPHABET.len())] as char)
        .collect();
    format!("{prefix}-{suffix}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_has_prefix() {
        let id = generate("bc");
        assert!(id.starts_with("bc-"), "id was: {id}");
    }

    #[test]
    fn generate_custom_prefix() {
        let id = generate("ma");
        assert!(id.starts_with("ma-"), "id was: {id}");
    }

    #[test]
    fn generate_correct_length() {
        let id = generate("bc");
        // "bc-" (3) + 3 chars = 6
        assert_eq!(id.len(), 6, "id was: {id}");
    }

    #[test]
    fn generate_suffix_is_alphanumeric() {
        let id = generate("bc");
        let suffix = id.split('-').nth(1).unwrap();
        assert!(
            suffix.chars().all(|c| c.is_ascii_alphanumeric()),
            "suffix was: {suffix}"
        );
    }

    #[test]
    fn generate_produces_unique_ids() {
        let ids: Vec<_> = (0..100).map(|_| generate("bc")).collect();
        let unique: std::collections::HashSet<_> = ids.iter().collect();
        assert!(
            unique.len() > 90,
            "too many collisions in 100 generated ids"
        );
    }
}
