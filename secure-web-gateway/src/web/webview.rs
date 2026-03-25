use std::path::PathBuf;
use tao::window::Window;
use wry::WebViewBuilder;

use super::super::UserEvent; 
use tao::event_loop::EventLoopProxy;

pub fn create_webview(
    window: &Window,
    proxy: EventLoopProxy<UserEvent>,
) -> wry::Result<wry::WebView> {
    
    let html_path: PathBuf = std::env::current_dir()
        .expect("Failed to get current directory")
        .join("assets")
        .join("index.html");

    println!("[debug] html path {:?}",html_path);

    let url = format!(
        "file:///{}",
        html_path.to_string_lossy().replace("\\", "/")
    );
    
    println!("[debug] file url: {}", url); 

    println!("[debug] Building webview...");

    let proxy_for_ipc = proxy.clone(); // Clone Arc for closure
    
    let webview = WebViewBuilder::new()
        .with_url(&url)
        .with_devtools(true)
        .with_ipc_handler(move |req| {
            println!("[debug] In IPC handler");
            let msg = req.body();
            println!("[debug webview]: json body {}",msg);
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(msg){
                if parsed["cmd"] == "navigate"{
                    if let Some(raw_input) = parsed["payload"].as_str() {
                        
                        println!("[ipc] Raw input from JS is {}", raw_input);
                        
                        let url = crate::web::navigation::resolve_input(raw_input);
                        
                        println!("[ipc] Resolved URL: {}", url);

                        if let Err(e) = proxy_for_ipc.send_event(UserEvent::Navigate(url)) {
                            eprintln!("[ipc] Event loop gone, could not send navigate {}", e);
                        }
                        else {
                            eprintln!("[ipc] Unknown message: {:?}", msg);
                        }
                    }
                
                }
            }
        })
        .build(window)?;

    Ok(webview)
}
