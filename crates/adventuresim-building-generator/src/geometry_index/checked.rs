//! Short-circuit geometry queries retain the first construction error.
//! A valid failed predicate and a malformed represented value remain distinct.

pub(crate) fn try_any<I, P, E>(iter: I, mut predicate: P) -> Result<bool, E>
where
    I: IntoIterator,
    P: FnMut(I::Item) -> Result<bool, E>,
{
    for value in iter {
        if predicate(value)? {
            return Ok(true);
        }
    }
    Ok(false)
}

pub(crate) fn try_all<I, P, E>(iter: I, mut predicate: P) -> Result<bool, E>
where
    I: IntoIterator,
    P: FnMut(I::Item) -> Result<bool, E>,
{
    for value in iter {
        if !predicate(value)? {
            return Ok(false);
        }
    }
    Ok(true)
}
