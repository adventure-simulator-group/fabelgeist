pub(super) const MAX_BYTES: usize = 63;

pub(super) const fn is_valid(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.is_empty() || bytes.len() > MAX_BYTES {
        return false;
    }

    let mut index = 0;
    while index < bytes.len() {
        let byte = bytes[index];
        if !(byte.is_ascii_lowercase()
            || byte.is_ascii_digit()
            || matches!(byte, b'_' | b'-' | b'.' | b':'))
        {
            return false;
        }
        index += 1;
    }
    true
}
