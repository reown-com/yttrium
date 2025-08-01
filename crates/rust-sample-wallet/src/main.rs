use {
    crate::app::App,
    leptos::prelude::*,
    thaw::{ConfigProvider, ToasterProvider},
    tracing_subscriber::fmt::MakeWriter,
    web_sys::{console, wasm_bindgen::JsValue},
};

mod app;
mod toast;

struct CustomConsoleWriter;

impl<'a> MakeWriter<'a> for CustomConsoleWriter {
    type Writer = CustomConsoleWriter;

    fn make_writer(&'a self) -> Self::Writer {
        CustomConsoleWriter
    }
}

impl std::io::Write for CustomConsoleWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let message = String::from_utf8_lossy(buf);
        console::log_1(&JsValue::from_str(&message));
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

fn main() {
    console_error_panic_hook::set_once();
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_writer(CustomConsoleWriter)
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
