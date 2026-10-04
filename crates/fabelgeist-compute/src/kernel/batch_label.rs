//! A borrowed diagnostic label for a batch's native command encoder.

/// Exact diagnostic spelling; it neither changes kernel identity nor cache
/// selection. Borrowing retains the native descriptor's original lifetime.
#[derive(Clone, Copy, Debug)]
pub struct KernelBatchLabel<'a>(&'a str);
impl<'a> From<&'a str> for KernelBatchLabel<'a> {
    fn from(label: &'a str) -> Self {
        Self(label)
    }
}
impl<'a> From<KernelBatchLabel<'a>> for &'a str {
    fn from(label: KernelBatchLabel<'a>) -> Self {
        label.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn borrowed_label_keeps_exact_diagnostic_spelling() {
        let spelling = String::from(" Δ \0");
        let label = KernelBatchLabel::from(spelling.as_str());
        assert_eq!(<&str>::from(label), spelling);
    }
}
