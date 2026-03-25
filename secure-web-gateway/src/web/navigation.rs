use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use url::Url;

pub fn resolve_input(input: &str) -> String {
    let trimmed = input.trim();

    println!("[[navigation] Trying to validate the url input");

    if let Ok(parsed) = Url::parse(trimmed){

        match parsed.scheme(){
            "http" | "https" => {
                println!("[navigation] Final resolved URL: {}", parsed);
                return parsed.to_string();
            }
        
            other => {
                eprintln!(
                    "[navigation] Blocked scheme '{}', treating as search query",
                    String::from(other)
                );
            }
        }
    }

    if trimmed.contains('.') && !trimmed.contains(' ') {
        let with_scheme = format!("https://{}", trimmed);
        if let Ok(parsed) = Url::parse(&with_scheme){
            if matches!(parsed.scheme(), "https"){
                println!("[navigation] Final resolved URL: {}", parsed);
                return parsed.to_string();
            }
        }
    }

    let encoded = utf8_percent_encode(trimmed, NON_ALPHANUMERIC).to_string();

    println!("[navigation] Final resolved URL (for search query): {}", encoded);
    format!("https://duckduckgo.com/?q={}", encoded)
    
}

#[cfg(test)]
mod tests {
    use super::resolve_input;

    #[test]
    fn passes_through_https_url() {
        assert_eq!(
            resolve_input("https://example.com"),
            "https://example.com/"  // url crate adds trailing slash to bare host
        );
    }

    #[test]
    fn passes_through_http_url() {
        assert_eq!(
            resolve_input("http://example.com/path"),
            "http://example.com/path"
        );
    }

    #[test]
    fn bare_domain_gets_https() {
        assert_eq!(
            resolve_input("example.com"),
            "https://example.com/"
        );
    }

    #[test]
    fn search_query_is_encoded() {
        let result = resolve_input("rust lang tutorial");
        assert!(result.starts_with("https://duckduckgo.com/?q="));
        // The space must be encoded
        assert!(result.contains("%20") || result.contains('+'));
        // The word "rust" must be present
        assert!(result.contains("rust"));
    }

    #[test]
    fn javascript_scheme_is_blocked() {
        let result = resolve_input("javascript:alert(1)");
        // Must NOT return the original JS URL
        assert!(!result.starts_with("javascript:"));
        // Must be a search query
        assert!(result.starts_with("https://duckduckgo.com/?q="));
    }

    #[test]
    fn data_scheme_is_blocked() {
        let result = resolve_input("data:text/html,<h1>pwned</h1>");
        assert!(!result.starts_with("data:"));
        assert!(result.starts_with("https://duckduckgo.com/?q="));
    }

    #[test]
    fn file_scheme_is_blocked() {
        let result = resolve_input("file:///etc/passwd");
        assert!(!result.starts_with("file:"));
        assert!(result.starts_with("https://duckduckgo.com/?q="));
    }

    #[test]
    fn ampersand_in_query_is_encoded() {
        let result = resolve_input("cats & dogs");
        // & must be encoded so it doesn't break the query string
        assert!(!result.contains("cats & dogs"));
        assert!(result.contains("%26") || !result.contains('&'));
    }
}