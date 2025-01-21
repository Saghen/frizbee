use memchr::{memchr, memchr2};

/// Ripped directly from nucleo-matcher. It makes the algo much faster but disables resistance
/// to typos
pub fn prefilter_ascii(needle: &[u8], mut haystack: &[u8]) -> Option<()> {
    // If the first char is later than the haystack.len() - needle.len() + 1, then the
    // haystack is too short to contain the needle
    let start = find_ascii_ignore_case(needle[0], &haystack[..haystack.len() - needle.len() + 1])?;
    haystack = &haystack[(start + 1)..];
    for &c in &needle[1..] {
        let idx = find_ascii_ignore_case(c, haystack)? + 1;
        haystack = &haystack[idx..];
    }
    Some(())
}

/// Same as prefilter_ascii but allows for a single typo
pub fn prefilter_ascii_with_typo(needle: &[u8], mut haystack: &[u8]) -> Option<()> {
    if needle.len() < 2 {
        return Some(());
    }

    // If the first char is later than the haystack.len() - needle.len() + 1, then the
    // haystack is too short to contain the needle. We use +2 instead of +1 to account
    // for the possibility of a single typo
    let (mut typos, start) = find_two_ascii_ignore_case(
        needle[0],
        needle[1],
        &haystack[..haystack.len() - needle.len() + 2],
    )?;
    haystack = &haystack[(start + 1)..];
    let mut idx = typos + 1;
    while idx < needle.len() {
        // Found a match
        if let Some(match_idx) = find_ascii_ignore_case(needle[idx], haystack) {
            haystack = &haystack[(match_idx + 1)..];
        } else {
            typos += 1;
            if typos > 1 {
                return None;
            }
        }
        idx += 1;
    }
    Some(())
}

#[inline(always)]
fn find_two_ascii_ignore_case(c1: u8, c2: u8, haystack: &[u8]) -> Option<(usize, usize)> {
    if let Some(idx) = find_ascii_ignore_case(c1, haystack) {
        return Some((0, idx));
    } else if let Some(idx) = find_ascii_ignore_case(c2, haystack) {
        return Some((1, idx));
    }
    None
}

#[inline(always)]
fn find_ascii_ignore_case(c: u8, haystack: &[u8]) -> Option<usize> {
    if c >= b'a' && c <= b'z' {
        memchr2(c, c - 32, haystack)
    } else {
        memchr(c, haystack)
    }
}
