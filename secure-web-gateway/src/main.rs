use tao::{
    event::{Event,WindowEvent},
    event_loop::{ControlFlow,EventLoop},
    window::WindowBuilder,
};

mod web;

fn main() -> wry::Result<()> {
    let event_loop = EventLoop::new();

    let window = WindowBuilder::new()
        .with_title("secure-web-gateway")
        .build(&event_loop)
        .expect("Failed to create window");

    let _webview = web::webview::create_webview(&window)?;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        if let Event::WindowEvent {event, ..} = event{
            if let WindowEvent::CloseRequested = event {
                *control_flow = ControlFlow::Exit;
            }
        }
    } );
}