use tao::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoopBuilder},
    window::WindowBuilder,
};
mod web;
use web::webview::create_webview;

#[derive(Debug, Clone)]
pub enum UserEvent {
    Navigate(String),
    NavigateBlocked,
}

fn main() {
    let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();
    let proxy = event_loop.create_proxy();

    let window = WindowBuilder::new()
        .with_title("secure-web-gateway")
        .build(&event_loop)
        .expect("Failed to create window");

    let webview = create_webview(&window, proxy)
        .expect("Failed to create webview");

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            
            Event::UserEvent(UserEvent::Navigate(url)) => {
                println!("[main] UserEvent::Navigate({})", url);
                
                
                let script = format!(
                    "document.getElementById('browser-frame').src = '{}';",
                    url.replace("'", "\\'")
                );
                
                if let Err(e) = webview.evaluate_script(&script) {
                    eprintln!("[main] Failed to inject navigate script: {}", e);
                }
            }
            
            Event::UserEvent(UserEvent::NavigateBlocked) => {
                println!("[main] UserEvent::NavigateBlocked");
                
                let script = "document.getElementById('browser-frame').src = 'asset://blocked.html';";
                
                if let Err(e) = webview.evaluate_script(script) {
                    eprintln!("[main] Failed to inject blocked script: {}", e);
                }
            }
            
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            
            _ => {}
        }
    });
}