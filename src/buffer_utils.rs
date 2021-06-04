/// Splits a `[u8]` array on another `[u8]` array.
///
/// Allows for splitting on more than one 'character',
/// much like string splitting.
pub fn split<'a>(buffer: &'a [u8], predicate: &[u8]) -> Vec<&'a [u8]> {
    let p_length = predicate.len();
    let mut parts: Vec<&[u8]> = Vec::new();

    let mut start = 0;
    let mut pos = find_substring(buffer, predicate).unwrap_or(0);

    while pos > start {
        if pos > start {
            parts.push(&buffer[start..pos]);
        }

        start = pos + p_length;
        pos = find_substring(&buffer[start..], predicate).unwrap_or(0) + start;
    }

    if start < buffer.len() {
        parts.push(&buffer[start..]);
    }

    parts
}

/// Finds the position of a 'substring' inside a buffer
pub fn find_substring(buffer: &[u8], substr: &[u8]) -> Option<usize> {
    buffer.windows(substr.len()).position(|win| win == substr)
}
