use super::openable;

#[test]
fn an_https_address_is_opened() {
    assert!(openable("https://github.com/tienduc120/cuakey"));
    assert!(openable("https://example.com"));
}

#[test]
fn anything_but_an_https_address_is_refused() {
    for url in [
        "http://example.com",
        "file:///C:/Windows/System32/calc.exe",
        r"C:\Windows\System32\calc.exe",
        "calc.exe",
        "javascript:alert(1)",
        "ms-settings:",
        "",
    ] {
        assert!(!openable(url), "{url:?}");
    }
}

#[test]
fn an_https_address_without_a_host_or_with_spaces_is_refused() {
    for url in [
        "https://",
        "https:///etc/passwd",
        " https://example.com",
        "https://exa mple.com",
        "https://example.com\n",
        "https://example.com\t--flag",
    ] {
        assert!(!openable(url), "{url:?}");
    }
}

#[cfg(windows)]
#[test]
fn open_url_refuses_before_the_shell_sees_the_address() {
    use super::{OpenUrlError, open_url};

    assert!(matches!(
        open_url(r"C:\Windows\System32\calc.exe"),
        Err(OpenUrlError::NotHttps)
    ));
}
