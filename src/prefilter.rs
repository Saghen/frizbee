use memchr::{memchr, memrchr};

/// Ripped directly from nucleo-matcher. It makes the algo much faster but also misses many
/// good matches
pub fn prefilter_ascii(needle: &[u8], mut haystack: &[u8]) -> Option<(usize, usize, usize)> {
    let start = memchr(needle[0], &haystack[..haystack.len() - needle.len() + 1])?;
    let mut greedy_end = start + 1;
    haystack = &haystack[greedy_end..];
    for &c in &needle[1..] {
        let idx = memchr(c, haystack)? + 1;
        greedy_end += idx;
        haystack = &haystack[idx..];
    }
    let end = greedy_end + memrchr(*needle.last().unwrap(), haystack).map_or(0, |i| i + 1);
    Some((start, greedy_end, end))
}
