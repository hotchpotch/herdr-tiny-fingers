pub const DEFAULT_ALPHABET: &str = "asdfqwerzxcvjklmiuopghtybn";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HintTarget<T> {
    pub hint: String,
    pub target: T,
}

pub fn assign_hints<T>(targets: Vec<T>) -> Vec<HintTarget<T>> {
    let hints = generate_hints(targets.len(), DEFAULT_ALPHABET);
    targets
        .into_iter()
        .zip(hints)
        .map(|(target, hint)| HintTarget { hint, target })
        .collect()
}

pub fn generate_hints(n: usize, alphabet: &str) -> Vec<String> {
    if n == 0 {
        return Vec::new();
    }
    let chars = alphabet.chars().collect::<Vec<_>>();
    assert!(!chars.is_empty(), "alphabet must not be empty");
    let mut width = 1;
    let mut capacity = chars.len();
    while capacity < n {
        width += 1;
        capacity *= chars.len();
    }
    (0..n)
        .map(|index| encode_fixed_width(index, width, &chars))
        .collect()
}

fn encode_fixed_width(mut index: usize, width: usize, alphabet: &[char]) -> String {
    let base = alphabet.len();
    let mut chars = vec![alphabet[0]; width];
    for slot in chars.iter_mut().rev() {
        *slot = alphabet[index % base];
        index /= base;
    }
    chars.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uses_single_letter_hints_when_alphabet_is_large_enough() {
        assert_eq!(generate_hints(5, "asdf"), ["aa", "as", "ad", "af", "sa"]);
        assert_eq!(generate_hints(4, "asdf"), ["a", "s", "d", "f"]);
    }

    #[test]
    fn fixed_width_after_overflow_keeps_hints_reachable() {
        let hints = generate_hints(5, "ab");
        assert_eq!(hints, ["aaa", "aab", "aba", "abb", "baa"]);
        assert!(hints.iter().all(|hint| hint.len() == 3));
    }
}
