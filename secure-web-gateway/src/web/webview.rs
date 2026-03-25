use std::path::PathBuf;
use tao::window::Window;
use wry::WebViewBuilder;

use super::super::UserEvent; 
use tao::event_loop::EventLoopProxy;
use web::naviagtion;

const IPC_SCHEME: &str = "ipc";
const EMPTY_BODY: &[u8] = b"";

pub fn create_webview(
    window: &Window,
    proxy: EventLoopProxy<UserEvent>,
) -> wry::Result<wry::WebView> {
    
    let html_path: PathBuf = std::env::current_dir()
        .expect("Failed to get current directory")
        .join("assets")
        .join("index.html");

    println!("[debug:webview] html path {:?}",html_path);

    let url = format!(
        "file:///{}",
        html_path.to_string_lossy().replace("\\", "/")
    );
    
    println!("[debug:webview] file url: {}", url); 

    println!("[debug:webview] Building webview...");

    let proxy_for_ipc = proxy.clone();
    
    let webview = WebViewBuilder::new()
        .with_url(&url)
        .with_devtools(true)
        .with_custom_protocol(IPC_SCHEME.to_string(), move |_webview_id, request| {
            // ✓ This closure must return Response directly
            handle_ipc_request(request, &proxy_for_ipc).unwrap_or_else(|e| {
                eprintln!("[ipc] Handler error: {}", e);
                error_response(500)
            })
        })
        .with_navigation_handler(|url| {
            println!("[nav] Requested navigation: {}", url);



        })
        .with_download_started_handler(|url, _path| {
            println!("[sandbox] blocked download: {}", url);
            false
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

fn error_response(status: u16) -> wry::http::Response<std::borrow::Cow<'static, [u8]>> {
    empty_response(status)
}

fn handle_ipc_request(
    request: wry::http::Request<Vec<u8>>,
    proxy: &EventLoopProxy<UserEvent>,
) -> Result<wry::http::Response<std::borrow::Cow<'static, [u8]>>, Box<dyn std::error::Error>> {
    // ✓ Internal function returns Result for error handling
    let uri = request.uri();
    let command = uri.path().trim_start_matches("/");

    println!("[ipc] Command: {:?}", command);

    match command {
        "navigate" => {
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
            println!("[ipc] Resolved URL: {:?}", url);

            if let Err(e) = proxy.send_event(UserEvent::Navigate(url)) {
                eprintln!("[ipc] Event loop gone: {}", e);
                return Ok(empty_response(500));
            }

            Ok(empty_response(200))
        }

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