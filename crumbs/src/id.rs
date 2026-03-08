use rand::RngExt;

const ALPHABET: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789";

/// Generate a unique random ID with the given prefix.
///
/// The suffix is 3 alphanumeric characters; format is `{prefix}-{3-char}`.
/// `is_taken` is called for each candidate; generation retries up to
/// `MAX_ATTEMPTS` times until a free ID is found.
pub fn generate(prefix: &str, mut is_taken: impl FnMut(&str) -> bool) -> anyhow::Result<String> {
    const MAX_ATTEMPTS: usize = 16;
    let mut rng = rand::rng();
    for _ in 0..MAX_ATTEMPTS {
        let suffix: String = (0..3)
            .map(|_| ALPHABET[rng.random_range(0..ALPHABET.len())] as char)
            .collect();
        let id = format!("{prefix}-{suffix}");
        if !is_taken(&id) {
            return Ok(id);
        }
    }
    anyhow::bail!("could not generate a unique ID after {MAX_ATTEMPTS} attempts")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_has_prefix() {
        let id = generate("bc", |_| false).unwrap();
        assert!(id.starts_with("bc-"), "id was: {id}");
    }

    #[test]
    fn generate_custom_prefix() {
        let id = generate("ma", |_| false).unwrap();
        assert!(id.starts_with("ma-"), "id was: {id}");
    }

    #[test]
    fn generate_correct_length() {
        let id = generate("bc", |_| false).unwrap();
        // "bc-" (3) + 3 chars = 6
        assert_eq!(id.len(), 6, "id was: {id}");
    }

    #[test]
    fn generate_suffix_is_alphanumeric() {
        let id = generate("bc", |_| false).unwrap();
        let suffix = id.split('-').nth(1).unwrap();
        assert!(
            suffix.chars().all(|c| c.is_ascii_alphanumeric()),
            "suffix was: {suffix}"
        );
    }

    #[test]
    fn generate_produces_unique_ids() {
        let mut seen = std::collections::HashSet::new();
        let ids: Vec<_> = (0..100)
            .map(|_| {
                let id = generate("bc", |c| seen.contains(c)).unwrap();
                seen.insert(id.clone());
                id
            })
            .collect();
        let unique: std::collections::HashSet<_> = ids.iter().collect();
        assert_eq!(unique.len(), 100, "got duplicate IDs");
    }

    #[test]
    fn generate_retries_on_collision() {
        // Reject the first attempt, accept subsequent ones.
        let mut calls = 0u32;
        let id = generate("bc", |_| {
            calls += 1;
            calls == 1 // reject only the very first candidate
        })
        .unwrap();
        assert!(id.starts_with("bc-"));
        assert!(calls >= 2, "expected at least 2 calls, got {calls}");
    }
}
