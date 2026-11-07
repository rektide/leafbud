//! Fractional-index key ordering — the read half of `fractional-indexing`.
//!
//! `root/page` and `card/block` positions are keys from the npm
//! `fractional-indexing` package (`generateKeyBetween`; keys like `a0`,
//! `a1V`, `a2G`). The library's contract is that **sort order equals
//! lexicographic order of the key strings** — that is the load-bearing
//! property for readers, and the only half extraction needs: we never mint
//! keys (no `generateKeyBetween`), we only order blocks by them
//! (`src/replicache/getBlocks.ts:77-81`: position asc, then fact id asc).
//!
//! Comparison is over UTF-8 bytes (the keys are ASCII base-62).

/// Lexicographic (byte-wise) comparison of fractional-index keys, matching
/// the npm library's ordering contract.
pub fn compare_keys(a: &str, b: &str) -> std::cmp::Ordering {
    a.as_bytes().cmp(b.as_bytes())
}

/// Sort `(position, fact id)` pairs the way `getBlocks.ts` orders
/// `card/block` facts: position ascending, fact id ascending on ties.
pub fn sort_by_position<T>(entries: &mut [(String, String, T)]) {
    entries.sort_by(|a, b| compare_keys(&a.1, &b.1).then_with(|| a.0.as_bytes().cmp(b.0.as_bytes())));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn npm_key_vectors_sort_lexicographically() {
        // Keys in npm-library generation order must come out of the
        // comparator in the same order.
        let mut keys = vec!["a2G", "a0", "a1V", "a1", "Zz", "a0Z"];
        keys.sort_by(|a, b| compare_keys(a, b));
        assert_eq!(keys, vec!["Zz", "a0", "a0Z", "a1", "a1V", "a2G"]);
    }

    #[test]
    fn position_then_fact_id_tiebreak() {
        let mut entries = vec![
            ("f3".to_string(), "a0".to_string(), 3),
            ("f1".to_string(), "a1".to_string(), 1),
            ("f2".to_string(), "a0".to_string(), 2),
        ];
        sort_by_position(&mut entries);
        let order: Vec<i32> = entries.iter().map(|e| e.2).collect();
        // Same position "a0": f2 < f3 by id.
        assert_eq!(order, vec![2, 3, 1]);
    }
}
