use alloc::borrow::{Cow, ToOwned};
use core::error::Error;
use core::fmt;
use core::ops::{Deref, Range};

use compact_str::CompactString;

use crate::{
    AbsPathBuf,
    InvalidNodeNameError,
    MAIN_SEPARATOR_CHAR,
    MAIN_SEPARATOR_STR,
    NodeName,
    r#const,
};

/// The borrowed version of [`AbsPathBuf`].
#[derive(Eq, PartialEq, Hash)]
#[repr(transparent)]
pub struct AbsPath(str);

/// TODO: docs.
pub struct Components<'path> {
    inner: &'path str,
}

/// TODO: docs.
#[cfg(feature = "std")]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum AbsPathFromPathError {
    /// The path is not absolute.
    NotAbsolute,

    /// The path is not valid UTF-8.
    NotUtf8,
}

/// TODO: docs.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct AbsPathNotAbsoluteError;

/// The type of error that can occur when [`normalizing`](AbsPath::normalize) a
/// path.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum NormalizeError {
    /// The path contains `..` components that would navigate above the root.
    EscapesRoot,

    /// The path component an invalid character at the given byte offset.
    InvalidCharacter { byte_offset: usize, ch: char },

    /// The path is not absolute.
    NotAbsolute,
}

struct NormalizeState<'a> {
    /// The offset in the original string up to which components have been
    /// processed. If it's less then the length of the original string, then
    /// it's guaranteed to be right after a path separator.
    cursor: usize,

    /// The normalized path being built. This is always a valid absolute path.
    normalized_path: NormalizedPath,

    /// The original string being normalized.
    original_str: &'a str,
}

enum NormalizedPath {
    Alloc(CompactString),
    /// A byte range in the [original string](NormalizeState::original_str)
    /// representing the path. Slicing with this range is guaranteed to return
    /// a valid absolute path.
    Slice(Range<usize>),
}

impl AbsPath {
    /// Returns the path as a string slice.
    #[inline]
    pub const fn as_str(&self) -> &str {
        &self.0
    }

    /// TODO: docs.
    #[inline]
    pub fn concat<'other>(&self, other: &'other Self) -> Cow<'other, Self> {
        if self == Self::root() {
            Cow::Borrowed(other)
        } else {
            Cow::Owned(self.to_owned().concat(other))
        }
    }

    /// TODO: docs.
    #[inline]
    pub const fn components(&self) -> Components<'_> {
        Components { inner: self.as_str() }
    }

    /// TODO: docs.
    #[inline]
    pub const fn from_str(
        str: &str,
    ) -> Result<&Self, AbsPathNotAbsoluteError> {
        let mut separator_offsets =
            r#const::str_char_offsets(str, MAIN_SEPARATOR_CHAR);

        let Some(offset) = separator_offsets.next() else {
            // The string doesn't contain the path separator.
            return Err(AbsPathNotAbsoluteError);
        };
        if offset != 0 {
            // The string doesn't start with the path separator.
            return Err(AbsPathNotAbsoluteError);
        }

        let separator_len = MAIN_SEPARATOR_STR.len();
        let mut valid_up_to = separator_len;

        while let Some(offset) = separator_offsets.next() {
            let component = r#const::str_slice(str, valid_up_to..offset);
            if NodeName::from_str(component).is_err() {
                // The string contains an invalid component.
                return Err(AbsPathNotAbsoluteError);
            }
            valid_up_to = offset + separator_len;
        }

        let last_component = r#const::str_slice(str, valid_up_to..str.len());
        if !last_component.is_empty()
            && NodeName::from_str(last_component).is_err()
        {
            // The string contains an invalid component.
            return Err(AbsPathNotAbsoluteError);
        }

        // SAFETY: just checked that the string is a valid absolute path.
        Ok(unsafe { AbsPath::from_str_unchecked(str) })
    }

    /// TODO: docs.
    #[inline]
    pub const fn is_root(&self) -> bool {
        debug_assert!(MAIN_SEPARATOR_STR.len() == 1);
        let bytes = self.as_str().as_bytes();
        bytes.len() == 1 && bytes[0] == MAIN_SEPARATOR_STR.as_bytes()[0]
    }

    /// TODO: docs.
    #[inline]
    pub fn join(&self, node_name: &NodeName) -> AbsPathBuf {
        let mut path = self.to_owned();
        path.push(node_name);
        path
    }

    /// TODO: docs.
    #[inline]
    pub const fn node_name(&self) -> Option<&NodeName> {
        self.components().next_back_const()
    }

    /// TODO: docs.
    #[inline]
    pub fn normalize(str: &str) -> Result<Cow<'_, Self>, NormalizeError> {
        let mut state = NormalizeState::new(str)?;
        while !state.process_component()? {}
        Ok(state.finish())
    }

    /// TODO: docs.
    #[inline]
    pub const fn parent(&self) -> Option<&Self> {
        let mut components = self.components();
        if components.next_back_const().is_some() {
            Some(components.as_path())
        } else {
            None
        }
    }

    /// TODO: docs.
    #[inline]
    pub const fn root() -> &'static Self {
        unsafe { Self::from_str_unchecked(MAIN_SEPARATOR_STR) }
    }

    /// TODO: docs.
    #[inline]
    pub const fn split_last(&self) -> Option<(&Self, &NodeName)> {
        let mut components = self.components();
        match components.next_back_const() {
            Some(last_component) => {
                Some((components.as_path(), last_component))
            },
            None => None,
        }
    }

    /// TODO: docs.
    #[inline]
    pub fn starts_with<P>(&self, base: P) -> bool
    where
        P: AsRef<AbsPath>,
    {
        let base = base.as_ref();

        base.is_root()
            || (self.as_str().starts_with(base.as_str())
                && self.as_str()[base.len()..]
                    .chars()
                    .next()
                    .map(|c| c == MAIN_SEPARATOR_CHAR)
                    .unwrap_or(true))
    }

    /// TODO: docs.
    #[inline]
    pub const fn strip_prefix<'this>(
        &'this self,
        other: &Self,
    ) -> Option<&'this Self> {
        if other.is_root() {
            return Some(self);
        }

        match r#const::str_strip_prefix(self.as_str(), other.as_str()) {
            Some(suffix) => {
                if suffix.is_empty() {
                    Some(Self::root())
                } else if r#const::str_starts_with_char(
                    suffix,
                    MAIN_SEPARATOR_CHAR,
                ) {
                    Some(unsafe { Self::from_str_unchecked(suffix) })
                } else {
                    None
                }
            },
            None => None,
        }
    }

    /// # Safety
    ///
    /// The caller must ensure that the given string is a valid absolute path.
    #[inline]
    pub const unsafe fn from_str_unchecked(s: &str) -> &Self {
        unsafe { &*(s as *const str as *const Self) }
    }
}

impl<'path> Components<'path> {
    /// TODO: docs.
    #[inline]
    pub const fn as_path(&self) -> &'path AbsPath {
        // SAFETY: the inner string is always a valid absolute path.
        unsafe { AbsPath::from_str_unchecked(self.inner) }
    }

    #[inline]
    const fn next_back_const(&mut self) -> Option<&'path NodeName> {
        let inner = self.inner;

        debug_assert!(r#const::str_starts_with_str(inner, MAIN_SEPARATOR_STR));

        if r#const::str_eq(inner, MAIN_SEPARATOR_STR) {
            return None;
        }

        debug_assert!(!r#const::str_ends_with_str(inner, MAIN_SEPARATOR_STR));

        let last_separator_offset = r#const::bytes_offset_of_last_occurrence(
            inner.as_bytes(),
            MAIN_SEPARATOR_CHAR as u8,
        )
        .expect("has separator");

        let (rest, component_with_leading_separator) =
            self.inner.split_at(last_separator_offset);

        let component = r#const::str_slice(
            component_with_leading_separator,
            1..component_with_leading_separator.len(),
        );

        self.inner = if !rest.is_empty() { rest } else { MAIN_SEPARATOR_STR };

        Some(unsafe { NodeName::from_str_unchecked(component) })
    }
}

impl<'a> NormalizeState<'a> {
    #[inline]
    fn finish(self) -> Cow<'a, AbsPath> {
        debug_assert!(self.cursor == self.original_str.len());
        match self.normalized_path {
            NormalizedPath::Alloc(str) => Cow::Owned(AbsPathBuf::new(str)),
            NormalizedPath::Slice(range) => {
                let str = &self.original_str[range];
                // SAFETY: the given range always slices a valid absolute path.
                Cow::Borrowed(unsafe { AbsPath::from_str_unchecked(str) })
            },
        }
    }

    #[inline]
    #[cfg(windows)]
    fn new(original_str: &'a str) -> Result<Self, NormalizeError> {
        if matches!(original_str.chars().next(), Some('a'..='z' | 'A'..='Z')) && &original_str[1..3] == ":\\" {
            let cursor = 4; // e.g. "C:\\"
            Ok(Self {
                cursor,
                normalized_path: NormalizedPath::Slice(0..cursor),
                original_str,
            })
        } else {
            Err(NormalizeError::NotAbsolute)
        }
    }

    #[inline]
    #[cfg(not(windows))]
    fn new(original_str: &'a str) -> Result<Self, NormalizeError> {
        if original_str.starts_with(MAIN_SEPARATOR_STR) {
            let cursor = MAIN_SEPARATOR_STR.len();
            Ok(Self {
                cursor,
                normalized_path: NormalizedPath::Slice(0..cursor),
                original_str,
            })
        } else {
            Err(NormalizeError::NotAbsolute)
        }
    }

    #[inline]
    fn process_component(&mut self) -> Result<bool, NormalizeError> {
        let (component_len, is_last_component) =
            match self.original_str.as_bytes()[self.cursor..]
                .iter()
                .position(|&b| b == MAIN_SEPARATOR_CHAR as u8)
            {
                Some(pos) => (pos, false),
                None => (self.original_str.len() - self.cursor, true),
            };

        Self::push_component(
            &mut self.normalized_path,
            self.original_str,
            self.cursor..self.cursor + component_len,
        )?;

        Ok(if is_last_component {
            self.cursor = self.original_str.len();
            true
        } else {
            self.cursor += component_len + MAIN_SEPARATOR_STR.len();
            self.cursor == self.original_str.len()
        })
    }

    #[inline]
    fn push_component(
        normalized_path: &mut NormalizedPath,
        original_str: &'a str,
        component_range: Range<usize>,
    ) -> Result<(), NormalizeError> {
        debug_assert!(component_range.end <= original_str.len(),);
        debug_assert!(
            original_str[..component_range.start]
                .ends_with(MAIN_SEPARATOR_CHAR)
        );

        let component = &original_str[component_range.clone()];

        let Err(err) = NodeName::from_str(component) else {
            match normalized_path {
                NormalizedPath::Alloc(str) => {
                    if str != MAIN_SEPARATOR_STR {
                        str.push_str(MAIN_SEPARATOR_STR);
                    }
                    str.push_str(component);
                },
                NormalizedPath::Slice(current_range) => {
                    if current_range.len() == MAIN_SEPARATOR_STR.len() {
                        *current_range = component_range;
                        current_range.start -= MAIN_SEPARATOR_STR.len();
                        return Ok(());
                    }

                    // If the component is an extension of the current string
                    // slice, we can avoid allocating.
                    if current_range.end + MAIN_SEPARATOR_STR.len()
                        == component_range.start
                    {
                        current_range.end = component_range.end;
                        return Ok(());
                    }

                    let mut new_path = CompactString::with_capacity(
                        current_range.len()
                            + MAIN_SEPARATOR_STR.len()
                            + component_range.len(),
                    );

                    new_path.push_str(&original_str[current_range.clone()]);
                    new_path.push_str(MAIN_SEPARATOR_STR);
                    new_path.push_str(component);

                    *normalized_path = NormalizedPath::Alloc(new_path);
                },
            }
            return Ok(());
        };

        match err {
            InvalidNodeNameError::Empty | InvalidNodeNameError::SingleDot => {
                return Ok(());
            },
            InvalidNodeNameError::ContainsInvalidCharacter(ch) => {
                return Err(NormalizeError::InvalidCharacter {
                    byte_offset: component_range.start,
                    ch,
                });
            },
            InvalidNodeNameError::DoubleDot => {},
        }

        let current_path = match normalized_path {
            NormalizedPath::Alloc(str) => &**str,
            NormalizedPath::Slice(range) => &original_str[range.clone()],
        };

        let offset_of_last_separator =
            r#const::bytes_offset_of_last_occurrence(
                current_path.as_bytes(),
                MAIN_SEPARATOR_CHAR as u8,
            )
            .ok_or(NormalizeError::EscapesRoot)?;

        let new_len = if offset_of_last_separator == 0 {
            if current_path.len() == MAIN_SEPARATOR_STR.len() {
                return Err(NormalizeError::EscapesRoot);
            } else {
                MAIN_SEPARATOR_STR.len()
            }
        } else {
            offset_of_last_separator
        };

        match normalized_path {
            NormalizedPath::Alloc(str) => {
                // SAFETY: the new length is less than the old length.
                unsafe {
                    str.set_len(new_len);
                }
            },
            NormalizedPath::Slice(range) => {
                range.end = range.start + new_len;
            },
        }

        Ok(())
    }
}

impl ToOwned for AbsPath {
    type Owned = AbsPathBuf;

    #[inline]
    fn to_owned(&self) -> Self::Owned {
        AbsPathBuf::new(self.as_str().into())
    }
}

impl Deref for AbsPath {
    type Target = str;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl AsRef<str> for AbsPath {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

#[cfg(feature = "std")]
impl AsRef<std::path::Path> for AbsPath {
    #[inline]
    fn as_ref(&self) -> &std::path::Path {
        std::path::Path::new(self.as_str())
    }
}

impl AsRef<Self> for AbsPath {
    #[inline]
    fn as_ref(&self) -> &Self {
        self
    }
}

impl fmt::Debug for AbsPath {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("AbsPath").field(&self.as_str()).finish()
    }
}

impl fmt::Display for AbsPath {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self.as_str(), f)
    }
}

impl PartialEq<AbsPathBuf> for AbsPath {
    #[inline]
    fn eq(&self, other: &AbsPathBuf) -> bool {
        self.as_str() == other.as_str()
    }
}

impl PartialEq<str> for AbsPath {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

impl PartialEq<AbsPath> for str {
    #[inline]
    fn eq(&self, other: &AbsPath) -> bool {
        other == self
    }
}

impl<'a> TryFrom<&'a str> for &'a AbsPath {
    type Error = AbsPathNotAbsoluteError;

    #[inline]
    fn try_from(str: &'a str) -> Result<Self, Self::Error> {
        <AbsPath>::from_str(str)
    }
}

#[cfg(feature = "std")]
impl<'a> TryFrom<&'a std::path::Path> for &'a AbsPath {
    type Error = AbsPathFromPathError;

    #[inline]
    fn try_from(path: &'a std::path::Path) -> Result<Self, Self::Error> {
        path.to_str().ok_or(AbsPathFromPathError::NotUtf8).and_then(|s| {
            Self::try_from(s).map_err(|_| AbsPathFromPathError::NotAbsolute)
        })
    }
}

impl<'path> Iterator for Components<'path> {
    type Item = &'path NodeName;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        debug_assert!(self.inner.starts_with(MAIN_SEPARATOR_STR));
        debug_assert!(
            !self.inner.ends_with(MAIN_SEPARATOR_STR)
                || self.inner == MAIN_SEPARATOR_STR
        );
        let s = &self.inner[1..];
        let separator = MAIN_SEPARATOR_CHAR as u8;
        let (component, rest) = match s.bytes().position(|b| b == separator) {
            Some(len) => s.split_at(len),
            None if !s.is_empty() => (s, MAIN_SEPARATOR_STR),
            None => return None,
        };
        self.inner = rest;
        Some(unsafe { NodeName::from_str_unchecked(component) })
    }
}

impl DoubleEndedIterator for Components<'_> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.next_back_const()
    }
}

impl fmt::Debug for Components<'_> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("Components").field(&self.as_path()).finish()
    }
}

#[cfg(feature = "std")]
impl fmt::Display for AbsPathFromPathError {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::NotAbsolute => AbsPathNotAbsoluteError.fmt(f),
            Self::NotUtf8 => f.write_str("path is not valid unicode"),
        }
    }
}

#[cfg(feature = "std")]
impl Error for AbsPathFromPathError {}

impl fmt::Display for AbsPathNotAbsoluteError {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("path is not absolute")
    }
}

impl Error for AbsPathNotAbsoluteError {}

impl fmt::Display for NormalizeError {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::EscapesRoot => {
                f.write_str("path escapes the root via `..` components")
            },
            Self::InvalidCharacter { byte_offset, ch } => {
                write!(
                    f,
                    "path contains invalid character '{ch}' at byte range \
                     {}..{}",
                    byte_offset,
                    byte_offset + ch.len_utf8(),
                )
            },
            Self::NotAbsolute => AbsPathNotAbsoluteError.fmt(f),
        }
    }
}

impl Error for NormalizeError {}

#[cfg(feature = "serde")]
mod serde_impls {
    use serde::de::{Deserialize, Deserializer, Error};
    use serde::ser::{Serialize, Serializer};

    use super::AbsPath;

    impl Serialize for AbsPath {
        #[inline]
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            self.as_str().serialize(serializer)
        }
    }

    impl<'de> Deserialize<'de> for &'de AbsPath {
        #[inline]
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            <&'de str>::deserialize(deserializer)?
                .try_into()
                .map_err(D::Error::custom)
        }
    }
}
