use {
    crate::app::App,
    leptos::prelude::*,
    thaw::{ConfigProvider, ToasterProvider},
};

mod app;
mod toast;

fn main() {
    console_error_panic_hook::set_once();
    tracing_subscriber::fmt()
        .with_writer(
            // To avoide trace events in the browser from showing their
            // JS backtrace, which is very annoying, in my opinion
            tracing_subscriber_wasm::MakeConsoleWriter::default()
                .map_trace_level_to(tracing::Level::DEBUG),
        )
        .with_max_level(tracing::Level::INFO)
        // For some reason, if we don't do this in the browser, we get
        // a runtime error.
        .without_time()
        .init();
    leptos::mount::mount_to_body(|| {
        view! {
            <ConfigProvider>
                <ToasterProvider>
                    <App />
                </ToasterProvider>
            </ConfigProvider>
        }
    })
}
