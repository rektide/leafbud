//! Fractional-index keys — ordering plus `generateKeyBetween` minting.
//!
//! `root/page` and `card/block` positions are keys from the npm
//! `fractional-indexing` package (Leaflet pins 3.2.0). The library's
//! contract is that **sort order equals lexicographic order of the key
//! strings** — the load-bearing property for readers
//! (`src/replicache/getBlocks.ts:77-81`: position asc, then fact id asc) —
//! and [`generate_key_between`] is the write half authoring needs: it mints
//! keys that sort strictly between two existing keys (appends after the
//! draft's last key never collide with concurrent editor inserts).
//!
//! The generator is a faithful port of fractional-indexing 3.2.0
//! `src/index.js` (the dgreensp observable implementation, CC0), verified
//! against vectors produced by running that exact package — see the tests.
//! Comparison is over UTF-8 bytes (the keys are ASCII base-62).

/// Base-62 digit alphabet (ascending char code order — required).
const BASE_62_DIGITS: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

/// Key-generation errors (invalid keys in, or the representable range out).
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum KeyError {
    #[error("invalid order key {key:?}: {reason}")]
    Invalid { key: String, reason: String },
    #[error("key {a:?} must sort before {b:?}")]
    Order { a: String, b: String },
    #[error("no key exists below {0:?} (smallest representable)")]
    CannotDecrement(String),
    #[error("no key exists above {0:?} (largest representable)")]
    CannotIncrement(String),
}

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

// ── generateKeyBetween (fractional-indexing 3.2.0 port) ─────────────────────

/// The integer-part length a key's head char encodes: `a`..`z` → 2..27
/// digits (positive), `A`..`Z` → 27..2 (negative).
fn integer_length(head: u8, key: &str) -> Result<usize, KeyError> {
    match head {
        b'a'..=b'z' => Ok((head - b'a') as usize + 2),
        b'A'..=b'Z' => Ok((b'Z' - head) as usize + 2),
        _ => Err(KeyError::Invalid {
            key: key.to_string(),
            reason: format!("invalid order key head {:?}", head as char),
        }),
    }
}

/// The integer part of a key (head + the digits it counts off).
fn integer_part(key: &str) -> Result<&str, KeyError> {
    let head = key.as_bytes().first().copied().ok_or_else(|| KeyError::Invalid {
        key: key.to_string(),
        reason: "empty key".to_string(),
    })?;
    let len = integer_length(head, key)?;
    if len > key.len() {
        return Err(KeyError::Invalid {
            key: key.to_string(),
            reason: "key shorter than its head declares".to_string(),
        });
    }
    Ok(&key[..len])
}

/// The npm package's validation: rejects the smallest representable key
/// itself, bad heads, too-short keys, and trailing-zero fractions.
fn validate_order_key(key: &str) -> Result<(), KeyError> {
    if key == "A00000000000000000000000000" {
        return Err(KeyError::Invalid {
            key: key.to_string(),
            reason: "this is the smallest representable key".to_string(),
        });
    }
    let integer = integer_part(key)?;
    if integer.len() != integer_length(integer.as_bytes()[0], key)? {
        return Err(KeyError::Invalid {
            key: key.to_string(),
            reason: "invalid integer part length".to_string(),
        });
    }
    // Trailing zeros are only invalid in the *fraction* (the integer part
    // legitimately ends in 0 — "a0" is the first key).
    if key[integer.len()..].ends_with('0') {
        return Err(KeyError::Invalid {
            key: key.to_string(),
            reason: "trailing zero in fraction".to_string(),
        });
    }
    Ok(())
}

fn digit_index(c: u8, key: &str) -> Result<usize, KeyError> {
    BASE_62_DIGITS.iter().position(|&d| d == c).ok_or_else(|| KeyError::Invalid {
        key: key.to_string(),
        reason: format!("char {:?} not in the base-62 alphabet", c as char),
    })
}

/// The midpoint of two fraction strings (`a` may be empty, `b` null);
/// fractional-indexing 3.2.0 `midpoint` port. Returns a fraction that
/// sorts strictly between them.
fn midpoint(a: &str, b: Option<&str>) -> Result<String, KeyError> {
    let zero = BASE_62_DIGITS[0];
    if let Some(b) = b
        && a.as_bytes() >= b.as_bytes()
    {
        return Err(KeyError::Order {
            a: a.to_string(),
            b: b.to_string(),
        });
    }
    let trailing_zero = |s: &str| s.as_bytes().last() == Some(&zero);
    if trailing_zero(a) || b.is_some_and(trailing_zero) {
        return Err(KeyError::Invalid {
            key: format!("{a:?}..{b:?}"),
            reason: "trailing zero".to_string(),
        });
    }
    if let Some(b) = b.filter(|b| !b.is_empty()) {
        // Remove the longest common prefix, padding `a` with 0s as we go
        // (`b` cannot end before `a` inside the common prefix).
        let (ab, bb) = (a.as_bytes(), b.as_bytes());
        let mut n = 0;
        while n < bb.len() && ab.get(n).copied().unwrap_or(zero) == bb[n] {
            n += 1;
        }
        if n > 0 {
            return Ok(format!("{}{}", &b[..n], midpoint(&a[n.min(a.len())..], Some(&b[n..]))?));
        }
    }
    // First digits (or lack of one) differ.
    let digit_a = if a.is_empty() { 0 } else { digit_index(a.as_bytes()[0], a)? };
    let digit_b = match b {
        Some(b) if !b.is_empty() => digit_index(b.as_bytes()[0], b)?,
        _ => BASE_62_DIGITS.len(),
    };
    if digit_b > digit_a + 1 {
        // Math.round of the mean (half rounds up, as JS does).
        let mid = (digit_a + digit_b).div_ceil(2);
        return Ok((BASE_62_DIGITS[mid] as char).to_string());
    }
    match b {
        Some(b) if b.len() > 1 => Ok(b[..1].to_string()),
        // `b` is null or a single digit: recurse past `a`'s first digit.
        _ => Ok(format!("{}{}", BASE_62_DIGITS[digit_a] as char, midpoint(&a[1.min(a.len())..], None)?,)),
    }
}

/// `x + 1` in the integer-part arithmetic; `None` past the largest key.
fn increment_integer(x: &str) -> Result<Option<String>, KeyError> {
    let head = x.as_bytes()[0];
    let mut digs: Vec<u8> = x.as_bytes()[1..].to_vec();
    let mut carry = true;
    let mut i = digs.len();
    while carry && i > 0 {
        i -= 1;
        let d = digit_index(digs[i], x)? + 1;
        if d == BASE_62_DIGITS.len() {
            digs[i] = BASE_62_DIGITS[0];
        } else {
            digs[i] = BASE_62_DIGITS[d];
            carry = false;
        }
    }
    if carry {
        if head == b'Z' {
            return Ok(Some(format!("a{}", BASE_62_DIGITS[0] as char)));
        }
        if head == b'z' {
            return Ok(None);
        }
        let h = head + 1;
        if h > b'a' {
            digs.push(BASE_62_DIGITS[0]);
        } else {
            digs.pop();
        }
        return Ok(Some(format!("{}{}", h as char, String::from_utf8_lossy(&digs))));
    }
    Ok(Some(format!("{}{}", head as char, String::from_utf8_lossy(&digs))))
}

/// `x - 1` in the integer-part arithmetic; `None` past the smallest key.
fn decrement_integer(x: &str) -> Result<Option<String>, KeyError> {
    let head = x.as_bytes()[0];
    let mut digs: Vec<u8> = x.as_bytes()[1..].to_vec();
    let mut borrow = true;
    let mut i = digs.len();
    while borrow && i > 0 {
        i -= 1;
        let d = digit_index(digs[i], x)? as isize - 1;
        if d == -1 {
            digs[i] = *BASE_62_DIGITS.last().expect("alphabet nonempty");
        } else {
            digs[i] = BASE_62_DIGITS[d as usize];
            borrow = false;
        }
    }
    if borrow {
        if head == b'a' {
            return Ok(Some(format!("Z{}", *BASE_62_DIGITS.last().expect("alphabet nonempty") as char)));
        }
        if head == b'A' {
            return Ok(None);
        }
        let h = head - 1;
        if h < b'Z' {
            digs.push(*BASE_62_DIGITS.last().expect("alphabet nonempty"));
        } else {
            digs.pop();
        }
        return Ok(Some(format!("{}{}", h as char, String::from_utf8_lossy(&digs))));
    }
    Ok(Some(format!("{}{}", head as char, String::from_utf8_lossy(&digs))))
}

/// Mint a fractional-index key sorting strictly between `a` (exclusive,
/// `None` = the start) and `b` (exclusive, `None` = the end) — the port of
/// the npm package's `generateKeyBetween` (3.2.0, base-62). Appending is
/// `generate_key_between(Some(last), None)`; mid-insert is
/// `generate_key_between(Some(lower), Some(upper))`.
pub fn generate_key_between(a: Option<&str>, b: Option<&str>) -> Result<String, KeyError> {
    if let Some(a) = a {
        validate_order_key(a)?;
    }
    if let Some(b) = b {
        validate_order_key(b)?;
    }
    if let (Some(a), Some(b)) = (a, b)
        && a.as_bytes() >= b.as_bytes()
    {
        return Err(KeyError::Order {
            a: a.to_string(),
            b: b.to_string(),
        });
    }
    match (a, b) {
        (None, None) => Ok(format!("a{}", BASE_62_DIGITS[0] as char)),
        (None, Some(b)) => {
            let ib = integer_part(b)?;
            // The 3.2.0 source has a smallest-integer special case here,
            // but its own validateOrderKey rejects that key as input, so
            // the branch is unreachable; integer_part + the walk below
            // cover everything that can reach this point.
            if ib.as_bytes() < b.as_bytes() {
                return Ok(ib.to_string());
            }
            decrement_integer(ib)?.ok_or_else(|| KeyError::CannotDecrement(b.to_string()))
        }
        (Some(a), None) => {
            let ia = integer_part(a)?;
            let fa = &a[ia.len()..];
            match increment_integer(ia)? {
                Some(next) => Ok(next),
                None => Ok(format!("{ia}{}", midpoint(fa, None)?)),
            }
        }
        (Some(a), Some(b)) => {
            let ia = integer_part(a)?;
            let fa = &a[ia.len()..];
            let ib = integer_part(b)?;
            if ia == ib {
                return Ok(format!("{ia}{}", midpoint(fa, Some(&b[ib.len()..]))?));
            }
            let i = increment_integer(ia)?.ok_or_else(|| KeyError::CannotIncrement(a.to_string()))?;
            if i.as_bytes() < b.as_bytes() {
                return Ok(i);
            }
            Ok(format!("{ia}{}", midpoint(fa, None)?))
        }
    }
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

    /// Vectors produced by running Leaflet's pinned fractional-indexing
    /// 3.2.0 (node, `generateKeyBetween` from the package tarball) — the
    /// port must reproduce them exactly. Regenerate against
    /// fractional-indexing@3.2.0 if this table ever disagrees.
    #[test]
    fn npm_320_vectors_reproduce_exactly() {
        let vectors: &[(Option<&str>, Option<&str>, &str)] = &[
            (None, None, "a0"),
            (None, Some("a0"), "Zz"),
            (None, Some("Zz"), "Zy"),
            (None, Some("a1"), "a0"),
            (None, Some("b00"), "az"),
            (None, Some("a0V"), "a0"),
            (Some("a0"), None, "a1"),
            (Some("a1"), None, "a2"),
            (Some("a9"), None, "aA"),
            (Some("aZ"), None, "aa"),
            (Some("az"), None, "b00"),
            (Some("b0z"), None, "b10"),
            (Some("Zz"), None, "a0"),
            (Some("Zy"), None, "Zz"),
            (Some("a0V"), None, "a1"),
            (Some("a01"), None, "a1"),
            (Some("a0"), Some("a1"), "a0V"),
            (Some("a0"), Some("a0V"), "a0G"),
            (Some("a0V"), Some("a1"), "a0l"),
            (Some("a1"), Some("a2"), "a1V"),
            (Some("a0"), Some("b00"), "a1"),
            (Some("a0V"), Some("a0W"), "a0VV"),
            (Some("az"), Some("b00"), "azV"),
            (Some("Zz"), Some("a0"), "ZzV"),
            (Some("Zz"), Some("ZzV"), "ZzG"),
            (Some("a0ZZ"), Some("a0a"), "a0Zn"),
        ];
        for (a, b, expect) in vectors {
            assert_eq!(&generate_key_between(*a, *b).unwrap(), expect, "between {a:?} and {b:?}");
        }
        // Inputs the package itself rejects as invalid order keys.
        for bad in ["Yy", "A00000000000000000000000000", "a000", "d000"] {
            assert!(generate_key_between(None, Some(bad)).is_err(), "{bad:?} must be invalid");
            assert!(generate_key_between(Some(bad), None).is_err(), "{bad:?} must be invalid");
        }
        // Inverted bounds.
        assert!(matches!(generate_key_between(Some("a1"), Some("a0")), Err(KeyError::Order { .. })));
        assert!(generate_key_between(Some("a0"), Some("a0")).is_err());
    }

    /// A 70-key append chain and a 12-step mid-insert halving chain, both
    /// produced by the same package run — exercises digit rollover,
    /// head-length growth (`az` → `b00`), and fraction deepening.
    #[test]
    fn npm_320_chains_reproduce_exactly() {
        let chain70 = [
            "a0", "a1", "a2", "a3", "a4", "a5", "a6", "a7", "a8", "a9", "aA", "aB", "aC", "aD", "aE", "aF", "aG", "aH", "aI", "aJ", "aK", "aL", "aM", "aN",
            "aO", "aP", "aQ", "aR", "aS", "aT", "aU", "aV", "aW", "aX", "aY", "aZ", "aa", "ab", "ac", "ad", "ae", "af", "ag", "ah", "ai", "aj", "ak", "al",
            "am", "an", "ao", "ap", "aq", "ar", "as", "at", "au", "av", "aw", "ax", "ay", "az", "b00", "b01", "b02", "b03", "b04", "b05", "b06", "b07",
        ];
        let mut cur: Option<String> = None;
        for expect in chain70 {
            let next = generate_key_between(cur.as_deref(), None).unwrap();
            assert_eq!(&next, expect);
            if let Some(prev) = &cur {
                assert!(prev.as_bytes() < next.as_bytes(), "{prev} < {next}");
            }
            cur = Some(next);
        }

        let mid12 = ["a0V", "a0G", "a08", "a04", "a02", "a01", "a00V", "a00G", "a008", "a004", "a002", "a001"];
        let mut hi = "a1".to_string();
        for expect in mid12 {
            let mid = generate_key_between(Some("a0"), Some(&hi)).unwrap();
            assert_eq!(&mid, expect);
            assert!("a0".as_bytes() < mid.as_bytes() && mid.as_bytes() < hi.as_bytes());
            hi = mid;
        }
    }

    /// The load-bearing authoring property: appends minted from an
    /// existing key (of any shape seen in real drafts — integer, fractional
    /// suffix) sort strictly after it.
    #[test]
    fn appends_always_sort_after_the_existing_last_key() {
        for last in ["a0", "a1", "a1V", "a2G", "a0Z", "Zz", "b00", "az", "a0l", "a004"] {
            let next = generate_key_between(Some(last), None).unwrap();
            assert!(last.as_bytes() < next.as_bytes(), "{last} < {next}");
        }
    }
}
