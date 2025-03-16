use alloc::borrow;
use core::{fmt, ops, str};

use smol_str::SmolStr;

use crate::{InvalidNodeNameError, NodeName};

/// TODO: docs.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NodeNameBuf {
    inner: SmolStr,
}

impl NodeNameBuf {
    /// TODO: docs.
    #[inline]
    pub fn as_str(&self) -> &str {
        self.inner.as_str()
    }

    #[inline]
    pub(crate) fn new(inner: SmolStr) -> Self {
        Self { inner }
    }
}

impl fmt::Debug for NodeNameBuf {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("NodeNameBuf").field(&self.as_str()).finish()
    }
}

impl fmt::Display for NodeNameBuf {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

impl ops::Deref for NodeNameBuf {
    type Target = NodeName;

    #[inline]
    fn deref(&self) -> &Self::Target {
        // SAFETY: `NodeNameBuf` and `NodeName` have the same invariants.
        unsafe { NodeName::from_str_unchecked(self.as_str()) }
    }
}

impl borrow::Borrow<NodeName> for NodeNameBuf {
    #[inline]
    fn borrow(&self) -> &NodeName {
        self
    }
}

impl AsRef<NodeName> for NodeNameBuf {
    #[inline]
    fn as_ref(&self) -> &NodeName {
        self
    }
}

impl str::FromStr for NodeNameBuf {
    type Err = InvalidNodeNameError;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        <&NodeName>::try_from(s).map(borrow::ToOwned::to_owned)
    }
}

#[cfg(feature = "serde")]
mod serde_impls {
    use serde::de::{Deserialize, Deserializer, Error};
    use serde::ser::{Serialize, Serializer};
    use smol_str::SmolStr;

    use super::NodeNameBuf;

    impl Serialize for NodeNameBuf {
        #[inline]
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            self.inner.serialize(serializer)
        }
    }

    impl<'de> Deserialize<'de> for NodeNameBuf {
        #[inline]
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            SmolStr::deserialize(deserializer)?
                .parse()
                .map_err(D::Error::custom)
        }
    }
}
