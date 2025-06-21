pub(crate) const fn bytes_eq(lhs: &[u8], rhs: &[u8]) -> bool {
    if lhs.len() != rhs.len() {
        return false;
    }
    let mut idx = 0;
    while idx < lhs.len() {
        if lhs[idx] != rhs[idx] {
            return false;
        }
        idx += 1;
    }
    true
}

pub(crate) const fn char_contains(needle: char, haystack: &[char]) -> bool {
    let mut idx = 0;
    while idx < haystack.len() {
        if haystack[idx] == needle {
            return true;
        }
        idx += 1;
    }
    false
}

/// Returns the first character of `str` that's included in `chars`, or `None`
/// if none of them are.
pub(crate) const fn char_find(str: &str, chars: &[char]) -> Option<char> {
    let mut iter = chars_iter(str);
    while let Some(char) = iter.next() {
        if char_contains(char, chars) {
            return Some(char);
        }
    }
    None
}

pub(crate) const fn chars_iter(str: &str) -> ConstChars<'_> {
    ConstChars { str }
}

pub(crate) const fn str_eq(lhs: &str, rhs: &str) -> bool {
    bytes_eq(lhs.as_bytes(), rhs.as_bytes())
}

/// Same as [`core::str::Chars`], but with a `const` [`next()`] method.
pub(crate) struct ConstChars<'str> {
    str: &'str str,
}

impl ConstChars<'_> {
    pub(crate) const fn next(&mut self) -> Option<char> {
        todo!();
    }
}
