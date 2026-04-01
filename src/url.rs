pub fn normalize_url(input: &str) -> String {
    let trimmed = input.trim();

    if trimmed.starts_with("about:") {
        return trimmed.to_string();
    }

    if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        return trimmed.to_string();
    }

    if trimmed.starts_with("localhost") || trimmed.starts_with("127.0.0.1") {
        return format!("http://{trimmed}");
    }

    format!("https://{trimmed}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_url_unchanged() {
        assert_eq!(normalize_url("https://example.com"), "https://example.com");
    }

    #[test]
    fn http_url_unchanged() {
        assert_eq!(normalize_url("http://example.com"), "http://example.com");
    }

    #[test]
    fn adds_https_to_domain() {
        assert_eq!(normalize_url("example.com"), "https://example.com");
    }

    #[test]
    fn adds_https_to_domain_with_path() {
        assert_eq!(
            normalize_url("example.com/page"),
            "https://example.com/page"
        );
    }

    #[test]
    fn trims_whitespace() {
        assert_eq!(
            normalize_url("  https://example.com  "),
            "https://example.com"
        );
    }

    #[test]
    fn localhost_with_port() {
        assert_eq!(normalize_url("localhost:3000"), "http://localhost:3000");
    }

    #[test]
    fn about_blank() {
        assert_eq!(normalize_url("about:blank"), "about:blank");
    }
}
