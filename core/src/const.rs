use core::ops::Range;
use core::slice;

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

/// Returns the offset of the last occurrence of the given byte in the slice,
/// or `None` if the slice doesn't contain it.
pub(crate) const fn bytes_offset_of_last_occurrence(
    bytes: &[u8],
    byte: u8,
) -> Option<usize> {
    let Some(mut idx) = bytes.len().checked_sub(1) else {
        return None;
    };
    loop {
        if bytes[idx] == byte {
            return Some(idx);
        }
        let Some(next_idx) = idx.checked_sub(1) else {
            return None;
        };
        idx = next_idx;
    }
}

pub(crate) const fn char_contains(needle: char, haystack: &[char]) -> bool {
    let mut idx = 0;
    while idx < haystack.len() {
        if needle == haystack[idx] {
            return true;
        }
        idx += 1;
    }
    false
}

pub(crate) const fn str_chars(str: &str) -> Chars<'_> {
    Chars { str }
}

/// Returns an iterator over the byte offsets of all the occurrences of the
/// given character in the string.
pub(crate) const fn str_char_offsets(str: &str, ch: char) -> CharOffsets<'_> {
    CharOffsets { char: ch, chars: str_chars(str), offset: 0 }
}

pub(crate) const fn str_eq(lhs: &str, rhs: &str) -> bool {
    bytes_eq(lhs.as_bytes(), rhs.as_bytes())
}

pub(crate) const fn str_ends_with_str(str: &str, suffix: &str) -> bool {
    if suffix.len() <= str.len() {
        str_eq(str_slice(str, str.len() - suffix.len()..str.len()), suffix)
    } else {
        false
    }
}

/// Returns the first character of `str` that's included in `chars`, or `None`
/// if none of them are.
pub(crate) const fn str_find_char(str: &str, chars: &[char]) -> Option<char> {
    let mut iter = str_chars(str);
    while let Some(char) = iter.next() {
        if char_contains(char, chars) {
            return Some(char);
        }
    }
    None
}

/// Returns the first code point in the given string, or `None` if the string
/// is empty.
///
/// Taken from [Nugine/const-str][1].
///
/// [1]: https://github.com/Nugine/const-str/blob/4618db23a5466414b0083519122a7dca36b4852c/const-str/src/utf8.rs#L182-L246
#[allow(clippy::many_single_char_names)]
pub(crate) const fn str_first_char(str: &str) -> Option<char> {
    const CONT_MASK: u8 = 0b0011_1111;

    const fn utf8_first_byte(byte: u8, width: u32) -> u32 {
        (byte & (0x7F >> width)) as u32
    }

    const fn utf8_acc_cont_byte(ch: u32, byte: u8) -> u32 {
        (ch << 6) | (byte & CONT_MASK) as u32
    }

    #[allow(clippy::manual_unwrap_or_default)] // FIXME
    const fn unwrap_or_0(opt: Option<u8>) -> u8 {
        match opt {
            Some(byte) => byte,
            None => 0,
        }
    }

    let bytes = str.as_bytes();

    let mut i = 0;

    macro_rules! next {
        () => {{
            if i < bytes.len() {
                let x = Some(bytes[i]);
                i += 1;
                x
            } else {
                None
            }
        }};
    }

    let x = match next!() {
        Some(x) => x,
        None => return None,
    };
    if x < 128 {
        return Some(x as char);
    }

    let init = utf8_first_byte(x, 2);
    let y = unwrap_or_0(next!());
    let mut ch = utf8_acc_cont_byte(init, y);
    if x >= 0xE0 {
        let z = unwrap_or_0(next!());
        let y_z = utf8_acc_cont_byte((y & CONT_MASK) as u32, z);
        ch = (init << 12) | y_z;
        if x >= 0xF0 {
            #[allow(unused_assignments)]
            let w = unwrap_or_0(next!());
            ch = ((init & 7) << 18) | utf8_acc_cont_byte(y_z, w);
        }
    }

    Some(char::from_u32(ch).expect("char is valid"))
}

pub(crate) const fn str_slice(str: &str, byte_range: Range<usize>) -> &str {
    assert!(str.is_char_boundary(byte_range.start));
    assert!(str.is_char_boundary(byte_range.end));

    unsafe { str::from_utf8_unchecked(subslice(str.as_bytes(), byte_range)) }
}

pub(crate) const fn str_starts_with_char(str: &str, ch: char) -> bool {
    matches!(str_first_char(str), Some(first_ch) if ch == first_ch)
}

pub(crate) const fn str_starts_with_str(str: &str, prefix: &str) -> bool {
    if prefix.len() <= str.len() {
        str_eq(str_slice(str, 0..prefix.len()), prefix)
    } else {
        false
    }
}

pub(crate) const fn str_strip_prefix<'str>(
    str: &'str str,
    prefix: &str,
) -> Option<&'str str> {
    if str_starts_with_str(str, prefix) {
        Some(str_slice(str, prefix.len()..str.len()))
    } else {
        None
    }
}

pub(crate) const fn subslice<T>(slice: &[T], range: Range<usize>) -> &[T] {
    assert!(range.start <= range.end);
    assert!(range.end <= slice.len());

    let ptr = slice.as_ptr();
    let range_len = range.end - range.start;
    unsafe { slice::from_raw_parts(ptr.add(range.start), range_len) }
}

/// Same as [`core::str::Chars`], but with a `const` [`next()`] method.
pub(crate) struct Chars<'str> {
    str: &'str str,
}

pub(crate) struct CharOffsets<'str> {
    char: char,
    chars: Chars<'str>,
    offset: usize,
}

impl Chars<'_> {
    pub(crate) const fn next(&mut self) -> Option<char> {
        match str_first_char(self.str) {
            Some(first_char) => {
                let slice_from = first_char.len_utf8();
                self.str = str_slice(self.str, slice_from..self.str.len());
                Some(first_char)
            },
            None => None,
        }
    }
}

impl CharOffsets<'_> {
    pub(crate) const fn next(&mut self) -> Option<usize> {
        match self.chars.next() {
            Some(char) => {
                let current_offset = self.offset;
                self.offset += char.len_utf8();
                if char == self.char {
                    Some(current_offset)
                } else {
                    self.next()
                }
            },
            None => None,
        }
    }
}
