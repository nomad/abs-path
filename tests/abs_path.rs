use abs_path::{AbsPath, AbsPathBuf, NodeName, path};

#[test]
fn components_empty() {
    let path = AbsPath::root();
    assert_eq!(path.components().next(), None);
    assert_eq!(path.components().next_back(), None);
}

#[test]
#[cfg_attr(target_os = "windows", ignore)]
fn components_1() {
    let path = <&AbsPath>::try_from("/baz.txt").unwrap();
    let mut components = path.components();
    assert_eq!(components.next().unwrap(), "baz.txt");
    assert_eq!(components.next(), None);
}

#[test]
#[cfg_attr(target_os = "windows", ignore)]
fn components_2() {
    let path = <&AbsPath>::try_from("/foo/bar/baz.txt").unwrap();
    let mut components = path.components();
    assert_eq!(components.next().unwrap(), "foo");
    assert_eq!(components.next().unwrap(), "bar");
    assert_eq!(components.next().unwrap(), "baz.txt");
    assert_eq!(components.next(), None);
}

#[test]
#[cfg_attr(target_os = "windows", ignore)]
fn components_rev_1() {
    let path = <&AbsPath>::try_from("/baz.txt").unwrap();
    let mut components = path.components();
    assert_eq!(components.next_back().unwrap(), "baz.txt");
    assert_eq!(components.next(), None);
}

#[test]
#[cfg_attr(target_os = "windows", ignore)]
fn components_rev_2() {
    let path = <&AbsPath>::try_from("/foo/bar/baz.txt").unwrap();
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
fn starts_with() {
    let p = <&AbsPath>::try_from("/foo/bar").unwrap();
    assert!(p.starts_with(AbsPath::root()));
    assert!(p.starts_with(path!("/foo")));
    assert!(p.starts_with(path!("/foo/bar")));
    assert!(!p.starts_with(path!("/foo/bar.rs")));
    assert!(!p.starts_with(path!("/foo/bar/baz")));
}
