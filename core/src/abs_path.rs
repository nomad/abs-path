use alloc::borrow::{Cow, ToOwned};
use core::error::Error;
use core::fmt;
use core::ops::Deref;

use crate::{AbsPathBuf, MAIN_SEPARATOR_CHAR, MAIN_SEPARATOR_STR, NodeName};

/// The borrowed version of [`AbsPathBuf`].
#[derive(Eq, PartialEq, Hash)]
#[repr(transparent)]
pub struct AbsPath(str);

/// TODO: docs.
pub struct Components<'path> {
    inner: &'path str,
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
    pub fn components(&self) -> Components<'_> {
        Components { inner: self.as_str() }
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
    pub fn node_name(&self) -> Option<&NodeName> {
        self.components().next_back()
    }

    /// TODO: docs.
    #[inline]
    pub fn parent(&self) -> Option<&Self> {
        let mut components = self.components();
        components.next_back().is_some().then_some(components.as_path())
    }

    /// TODO: docs.
    #[inline]
    pub const fn root() -> &'static Self {
        unsafe { Self::from_str_unchecked(MAIN_SEPARATOR_STR) }
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
    pub fn strip_prefix<'this>(
        &'this self,
        other: &Self,
    ) -> Option<&'this Self> {
        self.as_str().strip_prefix(other.as_str()).map(|prefix| {
            if prefix.is_empty() {
                Self::root()
            } else {
                unsafe { Self::from_str_unchecked(prefix) }
            }
        })
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
    pub fn as_path(&self) -> &'path AbsPath {
        // SAFETY: the inner string is always a valid absolute path.
        unsafe { AbsPath::from_str_unchecked(self.inner) }
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
        let mut components = str.match_indices(MAIN_SEPARATOR_CHAR);

        let Some((offset, _)) = components.next() else {
            // The string doesn't contain the path separator.
            return Err(AbsPathNotAbsoluteError);
        };
        if offset != 0 {
            // The string doesn't start with the path separator.
            return Err(AbsPathNotAbsoluteError);
        }

        let separator_len = MAIN_SEPARATOR_STR.len();
        let mut valid_up_to = separator_len;

        for (offset, _) in components {
            let component = &str[valid_up_to..offset];
            if <&NodeName>::try_from(component).is_err() {
                // The string contains an invalid component.
                return Err(AbsPathNotAbsoluteError);
            }
            valid_up_to = offset + separator_len;
        }

        let last_component = &str[valid_up_to..];
        if !last_component.is_empty()
            && <&NodeName>::try_from(last_component).is_err()
        {
            // The string contains an invalid component.
            return Err(AbsPathNotAbsoluteError);
        }

        // SAFETY: just checked that the string is a valid absolute path.
        Ok(unsafe { AbsPath::from_str_unchecked(str) })
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
        debug_assert!(self.inner.starts_with(MAIN_SEPARATOR_STR));
        debug_assert!(
            !self.inner.ends_with(MAIN_SEPARATOR_STR)
                || self.inner == MAIN_SEPARATOR_STR
        );
        let s = &self.inner[1..];
        let (rest, component) =
            match s.bytes().rev().position(|b| b == MAIN_SEPARATOR_CHAR as u8)
            {
                Some(len) => {
                    let (rest, component_with_leading_separator) =
                        self.inner.split_at(self.inner.len() - len - 1);
                    (rest, &component_with_leading_separator[1..])
                },
                None if !s.is_empty() => (MAIN_SEPARATOR_STR, s),
                None => return None,
            };
        self.inner = rest;
        Some(unsafe { NodeName::from_str_unchecked(component) })
    }
}

impl fmt::Debug for Components<'_> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("Components").field(&self.as_path()).finish()
    }
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

impl fmt::Display for AbsPathNotAbsoluteError {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("path is not absolute")
    }
}

#[cfg(feature = "std")]
impl Error for AbsPathFromPathError {}
impl Error for AbsPathNotAbsoluteError {}

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
