use alloc::borrow::{Borrow, ToOwned};
use core::ops::Deref;
use core::{fmt, str};

use compact_str::CompactString;

use crate::{AbsPath, AbsPathNotAbsoluteError, MAIN_SEPARATOR_STR, NodeName};

/// TODO: docs.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct AbsPathBuf {
    inner: CompactString,
}

impl AbsPathBuf {
    /// TODO: docs.
    #[inline]
    pub fn as_str(&self) -> &str {
        self.inner.as_str()
    }

    /// TODO: docs.
    #[inline]
    pub fn concat(&mut self, other: &AbsPath) -> &mut Self {
        for other_component in other.components() {
            self.push(other_component);
        }
        self
    }

    /// TODO: docs.
    #[inline]
    pub fn join(mut self, node_name: &NodeName) -> Self {
        self.push(node_name);
        self
    }

    /// TODO: docs.
    #[inline]
    pub fn pop(&mut self) -> bool {
        match self.parent().map(|parent| parent.len()) {
            Some(len) => {
                self.inner.truncate(len);
                true
            },
            None => false,
        }
    }

    /// TODO: docs.
    #[inline]
    pub fn push<T: AsRef<NodeName>>(&mut self, node_name: T) -> &mut Self {
        if !self.is_root() {
            self.inner.push_str(MAIN_SEPARATOR_STR);
        }
        self.inner.push_str(node_name.as_ref().as_str());
        self
    }

    /// TODO: docs.
    #[inline]
    pub const fn root() -> Self {
        AbsPathBuf { inner: CompactString::const_new(MAIN_SEPARATOR_STR) }
    }

    #[inline]
    pub(crate) fn new(inner: CompactString) -> Self {
        Self { inner }
    }
}

impl Deref for AbsPathBuf {
    type Target = AbsPath;

    #[inline]
    fn deref(&self) -> &Self::Target {
        // SAFETY: `AbsPathBuf` and `AbsPath` have the same invariants.
        unsafe { AbsPath::from_str_unchecked(self.as_str()) }
    }
}

impl Borrow<AbsPath> for AbsPathBuf {
    #[inline]
    fn borrow(&self) -> &AbsPath {
        self
    }
}

impl AsRef<str> for AbsPathBuf {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

#[cfg(feature = "std")]
impl AsRef<std::path::Path> for AbsPathBuf {
    #[inline]
    fn as_ref(&self) -> &std::path::Path {
        <AbsPath>::as_ref(self)
    }
}

impl AsRef<AbsPath> for AbsPathBuf {
    #[inline]
    fn as_ref(&self) -> &AbsPath {
        self
    }
}

impl<'a> FromIterator<&'a NodeName> for AbsPathBuf {
    #[inline]
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = &'a NodeName>,
    {
        let mut ret = Self::root();
        for component in iter {
            ret.push(component);
        }
        ret
    }
}

impl fmt::Debug for AbsPathBuf {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("AbsPathBuf").field(&self.as_str()).finish()
    }
}

impl fmt::Display for AbsPathBuf {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self.as_str(), f)
    }
}

impl str::FromStr for AbsPathBuf {
    type Err = AbsPathNotAbsoluteError;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        <&AbsPath>::try_from(s).map(ToOwned::to_owned)
    }
}

impl PartialEq<&str> for AbsPathBuf {
    #[inline]
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

impl PartialEq<AbsPathBuf> for str {
    #[inline]
    fn eq(&self, other: &AbsPathBuf) -> bool {
        self == other.as_str()
    }
}

impl PartialEq<&AbsPath> for AbsPathBuf {
    #[inline]
    fn eq(&self, other: &&AbsPath) -> bool {
        self.deref() == *other
    }
}

impl From<&AbsPath> for AbsPathBuf {
    #[inline]
    fn from(path: &AbsPath) -> Self {
        path.to_owned()
    }
}

impl TryFrom<&str> for AbsPathBuf {
    type Error = AbsPathNotAbsoluteError;

    #[inline]
    fn try_from(path: &str) -> Result<Self, Self::Error> {
        <&AbsPath>::try_from(path).map(Self::from)
    }
}

#[cfg(feature = "std")]
impl TryFrom<&std::path::Path> for AbsPathBuf {
    type Error = crate::AbsPathFromPathError;

    #[inline]
    fn try_from(path: &std::path::Path) -> Result<Self, Self::Error> {
        <&AbsPath>::try_from(path).map(Self::from)
    }
}

#[cfg(feature = "std")]
impl TryFrom<std::path::PathBuf> for AbsPathBuf {
    type Error = crate::AbsPathFromPathError;

    #[inline]
    fn try_from(path: std::path::PathBuf) -> Result<Self, Self::Error> {
        <&AbsPath>::try_from(&*path).map(Self::from)
    }
}

#[cfg(feature = "std")]
impl From<AbsPathBuf> for std::path::PathBuf {
    #[inline]
    fn from(path: AbsPathBuf) -> Self {
        Self::from(std::ffi::OsString::from(path))
    }
}

#[cfg(feature = "std")]
impl From<AbsPathBuf> for std::ffi::OsString {
    #[inline]
    fn from(path: AbsPathBuf) -> Self {
        path.inner.into()
    }
}

#[cfg(feature = "serde")]
mod serde_impls {
    use compact_str::CompactString;
    use serde::de::{Deserialize, Deserializer, Error};
    use serde::ser::{Serialize, Serializer};

    use super::AbsPathBuf;

    impl Serialize for AbsPathBuf {
        #[inline]
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            self.inner.serialize(serializer)
        }
    }

    impl<'de> Deserialize<'de> for AbsPathBuf {
        #[inline]
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            CompactString::deserialize(deserializer)?
                .parse()
                .map_err(D::Error::custom)
        }
    }
}
