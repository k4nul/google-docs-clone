pub(crate) fn constant_time_eq(left: &str, right: &str) -> bool {
    let left = left.as_bytes();
    let right = right.as_bytes();
    let compared_len = left.len().max(right.len());
    let mut diff = left.len() ^ right.len();

    for index in 0..compared_len {
        let left_byte = left.get(index).copied().unwrap_or(0);
        let right_byte = right.get(index).copied().unwrap_or(0);
        diff |= usize::from(left_byte ^ right_byte);
    }

    diff == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constant_time_eq_accepts_matching_values() {
        assert!(constant_time_eq("doc-token", "doc-token"));
    }

    #[test]
    fn constant_time_eq_rejects_different_values_with_same_length() {
        assert!(!constant_time_eq("doc-token-a", "doc-token-b"));
    }

    #[test]
    fn constant_time_eq_rejects_different_values_with_different_lengths() {
        assert!(!constant_time_eq("doc-token", "doc-token-extra"));
    }
}
