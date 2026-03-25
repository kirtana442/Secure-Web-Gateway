use tao::window::Window;
use wry::WebViewBuilder;

use super::super::UserEvent;
use tao::event_loop::EventLoopProxy;
use super::sandbox::sandbox_allow_navigation;

const IPC_SCHEME: &str = "ipc";
const ASSET_SCHEME: &str = "asset";
const EMPTY_BODY: &[u8] = b"";


fn serve_asset_file(
    request: &wry::http::Request<Vec<u8>>
) -> Result<wry::http::Response<std::borrow::Cow<'static, [u8]>>, Box<dyn std::error::Error>> {

    let uri = request.uri();

    let host = uri.host().unwrap_or("");
    let file_name = host.strip_prefix("asset.").unwrap_or(host);

    println!("[asset] Host: {}", host);
    println!("[asset] Serving: {}", file_name);

    let html_content = match file_name {
        "index.html" => {
            println!("[asset] Loading index.html");
            include_str!("../../assets/index.html")
        }
        "blocked.html" => {
            println!("[asset] Loading blocked.html");
            include_str!("../../assets/blocked.html")
        }
        "favicon.ico" => {
            println!("[asset] Loading favicon");
            return Ok(empty_response(204));
        }
        _ => {
            println!("[asset] File not allowed: {}", file_name);
            return Ok(error_response(403, "Forbidden"));
        }
    };

    Ok(
    wry::http::Response::builder()
        .status(200)
        .header("Content-Type", "text/html; charset=utf-8")
        .header("Access-Control-Allow-Origin", "*")
        .header("Access-Control-Allow-Headers", "*")
        .header("Access-Control-Allow-Methods", "*")
        .header(
            "Content-Security-Policy",
            "default-src 'self' asset: ipc: https: http: data: blob:; \
             script-src 'self' 'unsafe-inline' asset: ipc:; \
             style-src 'self' 'unsafe-inline'; \
             frame-src https: http: asset: ipc:;"
        )
        .body(std::borrow::Cow::Borrowed(html_content.as_bytes()))?
    )
}

fn handle_ipc_request(
    request: &wry::http::Request<Vec<u8>>,
    proxy: &EventLoopProxy<UserEvent>,
) -> Result<wry::http::Response<std::borrow::Cow<'static, [u8]>>, Box<dyn std::error::Error>> {
    let uri = request.uri();
    let command = uri.path().trim_start_matches('/');
    
    println!("[ipc] Command: {:?}", command);
    
    match command {
        "navigate" => handle_ipc_navigate(uri, proxy),
        "ping" => {
            println!("[ipc] ping received");
            Ok(empty_response(200))
        }
        unknown => {
            eprintln!("[ipc] Unknown command: {:?}", &unknown[..unknown.len().min(40)]);
            Ok(empty_response(404))
        }
    }
}

/// Handle navigate IPC call
/// 1. Extract query parameter
/// 2. Decode and trim
/// 3. Resolve URL in Rust (sandbox validation)
/// 4. Send UserEvent to load in iframe
fn handle_ipc_navigate(
    uri: &wry::http::Uri,
    proxy: &EventLoopProxy<UserEvent>,
) -> Result<wry::http::Response<std::borrow::Cow<'static, [u8]>>, Box<dyn std::error::Error>> {
    let raw_input = uri.query()
        .and_then(|q| {
            q.split('&')
                .find(|part| part.starts_with("q="))
                .map(|part| &part[2..])
        })
        .unwrap_or("");

    if raw_input.is_empty() {
        eprintln!("[ipc] navigate: empty q parameter");
        return Ok(empty_response(400));
    }

    let decoded = percent_encoding::percent_decode_str(raw_input)
        .decode_utf8()
        .unwrap_or_default();

    let trimmed = decoded.trim().to_string();
    println!("[ipc] Decoded input: {:?}", trimmed);

    if trimmed.is_empty() {
        return Ok(empty_response(400));
    }

    let url = crate::web::navigation::resolve_input(&trimmed);
    
    if !sandbox_allow_navigation(&url) {
        eprintln!("[ipc] Sandbox rejected URL: {}", url);
        if let Err(e) = proxy.send_event(UserEvent::NavigateBlocked) {
            eprintln!("[ipc] Event loop gone: {}", e);
            return Ok(empty_response(500));
        }
        return Ok(empty_response(403));
    }
    
    println!("[ipc] Resolved URL (validated): {:?}", url);

    if let Err(e) = proxy.send_event(UserEvent::Navigate(url)) {
        eprintln!("[ipc] Event loop gone: {}", e);
        return Ok(empty_response(500));
    }

    Ok(empty_response(200))
}

pub fn create_webview(
    window: &Window,
    proxy: EventLoopProxy<UserEvent>,
) -> wry::Result<wry::WebView> {
    
    // Clone proxy for each handler
    let proxy_for_ipc = proxy.clone();
    let proxy_for_navigation = proxy.clone();
    let proxy_for_newwindow = proxy.clone();

    let webview = WebViewBuilder::new()
        .with_url("asset://index.html")
        
        .with_devtools(true)
        
        .with_custom_protocol(IPC_SCHEME.to_string(), move |_webview_id, request| {
            handle_ipc_request(&request, &proxy_for_ipc).unwrap_or_else(|e| {
                eprintln!("[ipc] Handler error: {}", e);
                error_response(500, "Internal Server Error")
            })
        })
        
        .with_custom_protocol(ASSET_SCHEME.to_string(), move |_webview_id, request| {
            serve_asset_file(&request).unwrap_or_else(|e| {
                eprintln!("[asset] Handler error: {}", e);
                error_response(500, "Internal Server Error")
            })
        })
        
        .with_navigation_handler(move |url| {
            println!("[nav-handler] URL attempted: {}", url);
            
            
            if url.contains("asset.") {
                println!("[nav-handler]  Asset protocol allowed");
                return true;
            }
            
            if url.starts_with("ipc://") {
                println!("[nav-handler]  IPC protocol allowed");
                return true;
            }
            
            let allowed = sandbox_allow_navigation(&url);
            if !allowed {
                eprintln!("[nav-handler]  Navigation blocked: {}", url);
                let _ = proxy_for_navigation.send_event(UserEvent::NavigateBlocked);
            }
            
            allowed
        })

        .with_new_window_req_handler(move |url, _target_hint| {
            println!("[new-window] Popup attempt: {}", url);
            
            if sandbox_allow_navigation(&url) {
                println!("[new-window]  Allowed");
                wry::NewWindowResponse::Allow
            } else {
                println!("[new-window]  Denied");
                // Optional: Send blocked event
                let _ = proxy_for_newwindow.send_event(UserEvent::NavigateBlocked);
                wry::NewWindowResponse::Deny
            }
        })

        .with_download_started_handler(|url, _path| {
            match url::Url::parse(&url) {
                Ok(parsed) => {
                    match parsed.scheme() {
                        "https" => {
                            println!("[download]  HTTPS download allowed: {}", url);
                            true
                        }
                        "blob" => {
                            println!("[download]  Blob download allowed: {}", url);
                            true
                        }
                        other => {
                            println!("[download]  Scheme {} blocked: {}", other, url);
                            false
                        }
                    }
                }
                Err(_) => {
                    println!("[download]  Invalid URL: {}", url);
                    false
                }
            }
        })

        .build(window)?;

    Ok(webview)
}


fn empty_response(status: u16) -> wry::http::Response<std::borrow::Cow<'static, [u8]>> {
    wry::http::Response::builder()
        .status(status)
        .header("Access-Control-Allow-Origin", "*")
        .body(std::borrow::Cow::Borrowed(EMPTY_BODY))
        .expect("Failed to build response")
}

fn error_response(status: u16, message: &'static str) -> wry::http::Response<std::borrow::Cow<'static, [u8]>> {
    wry::http::Response::builder()
        .status(status)
        .header("Access-Control-Allow-Origin", "*")
        .header("Content-Type", "text/plain")
        .body(std::borrow::Cow::Borrowed(message.as_bytes()))
        .expect("Failed to build response")
}