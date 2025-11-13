use std::borrow::Cow;

use abs_path::{AbsPath, AbsPathBuf, NodeName, NormalizeError, path};

#[test]
fn components_empty() {
    let path = AbsPath::root();
    assert_eq!(path.components().next(), None);
    assert_eq!(path.components().next_back(), None);
}

#[test]
#[cfg_attr(target_os = "windows", ignore)]
fn components_1() {
    let path = path!("/baz.txt");
    let mut components = path.components();
    assert_eq!(components.next().unwrap(), "baz.txt");
    assert_eq!(components.next(), None);
}

#[test]
#[cfg_attr(target_os = "windows", ignore)]
fn components_2() {
    let path = path!("/foo/bar/baz.txt");
    let mut components = path.components();
    assert_eq!(components.next().unwrap(), "foo");
    assert_eq!(components.next().unwrap(), "bar");
    assert_eq!(components.next().unwrap(), "baz.txt");
    assert_eq!(components.next(), None);
}

#[test]
#[cfg_attr(target_os = "windows", ignore)]
fn components_rev_1() {
    let path = path!("/baz.txt");
    let mut components = path.components();
    assert_eq!(components.next_back().unwrap(), "baz.txt");
    assert_eq!(components.next(), None);
}

#[test]
#[cfg_attr(target_os = "windows", ignore)]
fn components_rev_2() {
    let path = path!("/foo/bar/baz.txt");
    let mut components = path.components();
    assert_eq!(components.next_back().unwrap(), "baz.txt");
    assert_eq!(components.next_back().unwrap(), "bar");
    assert_eq!(components.next_back().unwrap(), "foo");
    assert_eq!(components.next_back(), None);
}

#[test]
fn from_iter_empty() {
    let path: AbsPathBuf = core::iter::empty::<&NodeName>().collect();
    assert_eq!(path, AbsPath::root());
}

#[test]
#[cfg_attr(target_os = "windows", ignore)]
fn from_single_separator() {
    assert_eq!(<&AbsPath>::try_from("/").unwrap(), AbsPath::root());
}

#[test]
#[cfg_attr(target_os = "windows", ignore)]
fn from_iter_1() {
    let path: AbsPathBuf = ["foo", "bar", "baz.txt"]
        .into_iter()
        .map(|s| <&NodeName>::try_from(s).unwrap())
        .collect();

    assert_eq!(path, "/foo/bar/baz.txt");
}

#[test]
#[cfg_attr(target_os = "windows", ignore)]
fn normalize_1() {
    let p = "/foo/..";
    assert_eq!(AbsPath::normalize(p), Ok(Cow::Borrowed(path!("/"))));
}

#[test]
#[cfg_attr(target_os = "windows", ignore)]
fn normalize_2() {
    let p = "/foo/.";
    assert_eq!(AbsPath::normalize(p), Ok(Cow::Borrowed(path!("/foo"))));
}

#[test]
#[cfg_attr(target_os = "windows", ignore)]
fn normalize_3() {
    let p = "/foo//bar";
    assert_eq!(AbsPath::normalize(p).as_deref(), Ok(path!("/foo/bar")));
}

#[test]
#[cfg_attr(target_os = "windows", ignore)]
fn normalize_4() {
    let p = "/foo/bar/../baz/";
    assert_eq!(AbsPath::normalize(p).as_deref(), Ok(path!("/foo/baz")));
}

#[test]
#[cfg_attr(target_os = "windows", ignore)]
fn normalize_5() {
    let p = "/.";
    assert_eq!(AbsPath::normalize(p).as_deref(), Ok(path!("/")));
}

#[test]
#[cfg_attr(target_os = "windows", ignore)]
fn normalize_6() {
    let p = "/foo/../bar/.//baz";
    assert_eq!(AbsPath::normalize(p).as_deref(), Ok(path!("/bar/baz")));
}

#[test]
#[cfg_attr(target_os = "windows", ignore)]
fn normalize_7() {
    let p = "/../foo";
    assert_eq!(AbsPath::normalize(p), Err(NormalizeError::EscapesRoot));
}

#[test]
#[cfg_attr(target_os = "windows", ignore)]
fn normalize_8() {
    let p = "/foo//";
    assert_eq!(AbsPath::normalize(p), Ok(Cow::Borrowed(path!("/foo"))));
}

#[test]
#[cfg_attr(target_os = "windows", ignore)]
fn normalize_9() {
    let p = "/.";
    assert_eq!(AbsPath::normalize(p), Ok(Cow::Borrowed(path!("/"))));
}

#[test]
#[cfg_attr(target_os = "windows", ignore)]
fn normalize_10() {
    let p = "//foo/bar";
    assert_eq!(AbsPath::normalize(p), Ok(Cow::Borrowed(path!("/foo/bar"))));
}

#[test]
#[cfg_attr(target_os = "windows", ignore)]
fn normalize_11() {
    let p = "/./foo/bar";
    assert_eq!(AbsPath::normalize(p), Ok(Cow::Borrowed(path!("/foo/bar"))));
}

#[test]
#[cfg_attr(target_os = "windows", ignore)]
fn normalize_12() {
    let p = "/foo/../bar/baz";
    assert_eq!(AbsPath::normalize(p), Ok(Cow::Borrowed(path!("/bar/baz"))));
}

#[test]
#[cfg_attr(target_os = "windows", ignore)]
fn starts_with() {
    let p = path!("/foo/bar");
    assert!(p.starts_with(AbsPath::root()));
    assert!(p.starts_with(path!("/foo")));
    assert!(p.starts_with(path!("/foo/bar")));
    assert!(!p.starts_with(path!("/foo/bar.rs")));
    assert!(!p.starts_with(path!("/foo/bar/baz")));
}

#[test]
#[cfg_attr(target_os = "windows", ignore)]
fn strip_prefix_root() {
    let p = path!("/foo/bar");
    assert_eq!(p.strip_prefix(AbsPath::root()).unwrap(), "/foo/bar");
}
