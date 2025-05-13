use alloc::borrow;
use core::{error, fmt, ops};

use crate::NodeNameBuf;

const INVALID_CHARACTERS: &[char] = &['/', '\\', '\0', '\n', '\r'];

/// The borrowed version of [`NodeNameBuf`].
#[derive(Eq, PartialEq, Hash)]
#[repr(transparent)]
pub struct NodeName(str);

impl NodeName {
    /// TODO: docs.
    #[inline]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// # Safety
    ///
    /// The caller must ensure that the given string is a valid file name.
    #[inline]
    pub const unsafe fn from_str_unchecked(s: &str) -> &Self {
        unsafe { &*(s as *const str as *const Self) }
    }
}

impl fmt::Debug for NodeName {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("NodeName").field(&self.as_str()).finish()
    }
}

impl fmt::Display for NodeName {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

impl PartialEq<str> for NodeName {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

impl PartialEq<NodeName> for str {
    #[inline]
    fn eq(&self, other: &NodeName) -> bool {
        self == other.as_str()
    }
}

impl borrow::ToOwned for NodeName {
    type Owned = NodeNameBuf;

    #[inline]
    fn to_owned(&self) -> Self::Owned {
        NodeNameBuf::new(self.as_str().into())
    }
}

impl AsRef<Self> for NodeName {
    #[inline]
    fn as_ref(&self) -> &Self {
        self
    }
}

impl AsRef<str> for NodeName {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl<'a> TryFrom<&'a str> for &'a NodeName {
    type Error = InvalidNodeNameError;

    #[inline]
    fn try_from(s: &'a str) -> Result<Self, Self::Error> {
        if s.is_empty() {
            Err(InvalidNodeNameError::Empty)
        } else if s == "." {
            Err(InvalidNodeNameError::SingleDot)
        } else if s == ".." {
            Err(InvalidNodeNameError::DoubleDot)
        } else if let Some(invalid) =
            s.chars().find(|c| INVALID_CHARACTERS.contains(c))
        {
            Err(InvalidNodeNameError::ContainsInvalidCharacter(invalid))
        } else {
            // SAFETY: checked above.
            Ok(unsafe { NodeName::from_str_unchecked(s) })
        }
    }
}

impl ops::Deref for NodeName {
    type Target = str;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

/// The error type returned when converting a string to a [`NodeName`] fails.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum InvalidNodeNameError {
    /// The file name is empty.
    Empty,

    /// The file name is a single dot (`.`), which is used to refer to the
    /// current directory.
    SingleDot,

    /// The file name is a double dot (`..`), which is used to refer to the
    /// parent directory.
    DoubleDot,

    /// The file name contains an invalid character.
    ContainsInvalidCharacter(char),
}

impl fmt::Display for InvalidNodeNameError {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Empty => write!(f, "file name is empty"),
            Self::SingleDot => write!(f, "file name is a single dot (`.`)"),
            Self::DoubleDot => write!(f, "file name is a double dot (`..`)"),
            Self::ContainsInvalidCharacter(c) => {
                write!(f, "file name contains an invalid character: {c:?}")
            },
        }
    }
}

impl error::Error for InvalidNodeNameError {}

#[cfg(feature = "serde")]
mod serde_impls {
    use serde::de::{Deserialize, Deserializer, Error};
    use serde::ser::{Serialize, Serializer};

    use super::NodeName;

    impl Serialize for NodeName {
        #[inline]
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            self.as_str().serialize(serializer)
        }
    }

    impl<'de> Deserialize<'de> for &'de NodeName {
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
