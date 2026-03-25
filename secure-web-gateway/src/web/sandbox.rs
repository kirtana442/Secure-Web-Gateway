use url::Url;

pub fn sandbox_allow_navigation(url: &str) -> bool {
    match Url::parse(url) {
        Ok(parsed) => {

            let scheme = parsed.scheme();

            match scheme {

                
                "http" | "https" => {
                    println!("[sandbox] allowed network: {}", url);
                    true
                }

                
                "ws" | "wss" => {
                    println!("[sandbox] websocket allowed: {}", url);
                    true
                }

                
                "about" => {
                    println!("[sandbox] about page: {}", url);
                    true
                }

                
                "blob" => {
                    println!("[sandbox] blob allowed: {}", url);
                    true
                }

                
                "data" => {
                    println!("[sandbox] data url (restricted): {}", url);

                    // optional: restrict to images only
                    if url.starts_with("data:image") {
                        true
                    } else {
                        false
                    }
                }

                
                "javascript" => {
                    println!("[sandbox] blocked javascript: {}", url);
                    false
                }

                "file" => {
                    println!("[sandbox] blocked file access: {}", url);
                    false
                }

                "ftp" => {
                    println!("[sandbox] blocked ftp: {}", url);
                    false
                }

                "view-source" => {
                    println!("[sandbox] blocked view-source: {}", url);
                    false
                }

                
                "mailto" | "tel" | "sms" => {
                    println!("[sandbox] blocked external app scheme: {}", url);
                    false
                }

                
                other => {
                    println!(
                        "[sandbox] unknown scheme '{}' blocked by default",
                        other
                    );
                    false
                }
            }
        }

        Err(_) => {
            println!("[sandbox] invalid url: {}", url);
            false
        }
    }
}