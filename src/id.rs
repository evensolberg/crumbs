use rand::RngExt;

const ALPHABET: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789";

pub fn generate() -> String {
    let mut rng = rand::rng();
    let suffix: String = (0..3)
        .map(|_| ALPHABET[rng.random_range(0..ALPHABET.len())] as char)
        .collect();
    format!("bc-{suffix}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_has_bc_prefix() {
        let id = generate();
        assert!(id.starts_with("bc-"), "id was: {id}");
    }

    #[test]
    fn generate_correct_length() {
        let id = generate();
        // "bc-" (3) + 3 chars = 6
        assert_eq!(id.len(), 6, "id was: {id}");
    }

    #[test]
    fn generate_suffix_is_alphanumeric() {
        let id = generate();
        let suffix = &id[3..];
        assert!(
            suffix.chars().all(|c| c.is_ascii_alphanumeric()),
            "suffix was: {suffix}"
        );
    }

    #[test]
    fn generate_produces_unique_ids() {
        let ids: Vec<_> = (0..100).map(|_| generate()).collect();
        let unique: std::collections::HashSet<_> = ids.iter().collect();
        // With 36^3 = 46656 possibilities, 100 should almost certainly be unique
        assert!(unique.len() > 90, "too many collisions in 100 generated ids");
    }
}
