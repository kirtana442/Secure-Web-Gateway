use std::path::PathBuf;
use tao::window::Window;
use wry::WebViewBuilder;

/// Create and configure the application's WebView.
///
/// Security model:
/// - WebView is treated as untrusted
/// - Only local static content is loaded at this stage

pub fn create_webview(window: &Window) -> wry::Result<wry::WebView>
{
    let html_path: PathBuf = std::env::current_dir()
        .expect("Failed to get current directory")
        .join("assets")
        .join("index.html");

    let url = format!("file://{}", html_path.display());

    WebViewBuilder::new()
        .with_url(&url)
        .build(window)
}