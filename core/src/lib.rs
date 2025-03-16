//! TODO: docs.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

mod abs_path;
mod abs_path_buf;
mod node_name;
mod node_name_buf;

#[cfg(feature = "std")]
pub use abs_path::AbsPathFromPathError;
pub use abs_path::{AbsPath, AbsPathNotAbsoluteError, Components};
pub use abs_path_buf::AbsPathBuf;
pub use node_name::{InvalidNodeNameError, NodeName};
pub use node_name_buf::NodeNameBuf;

#[cfg(not(windows))]
const MAIN_SEPARATOR_CHAR: char = '/';

#[cfg(windows)]
const MAIN_SEPARATOR_CHAR: char = '\\';

#[cfg(not(windows))]
const MAIN_SEPARATOR_STR: &str = "/";

#[cfg(windows)]
const MAIN_SEPARATOR_STR: &str = "\\";
