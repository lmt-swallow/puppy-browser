//! This module includes misc implementations.

use std::path::PathBuf;

/// `normalize_fileurl_with` resolves the relative path in the URL with `file://` scheme.
pub fn normalize_fileurl_with(mut basedir: PathBuf, u: String) -> String {
    if u.starts_with("http://") || u.starts_with("https://") {
        u
    } else {
        if u.starts_with("/") {
            format!("file://{}", u)
        } else {
            basedir.push(u);
            format!("file://{}", basedir.to_str().unwrap())
        }
    }
}

#[test]
fn test_normalize_fileurl_with() {
    let pbuf = PathBuf::from("/tmp");
    assert_eq!(
        normalize_fileurl_with(pbuf.clone(), "http://example.com".to_string()),
        "http://example.com"
    );
    assert_eq!(
        normalize_fileurl_with(pbuf.clone(), "https://example.com/path/path2".to_string()),
        "https://example.com/path/path2"
    );
    assert_eq!(
        normalize_fileurl_with(pbuf.clone(), "/etc/passwd".to_string()),
        "file:///etc/passwd"
    );
    assert_eq!(
        normalize_fileurl_with(pbuf.clone(), "test/aa.html".to_string()),
        "file:///tmp/test/aa.html"
    );
}
