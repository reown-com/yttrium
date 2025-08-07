use {std::sync::Arc, tracing_subscriber::fmt::MakeWriter};

#[derive(Clone)]
struct UniffiLogger(Arc<dyn Logger>);

impl<'a> MakeWriter<'a> for UniffiLogger {
    type Writer = UniffiLogger;

    fn make_writer(&'a self) -> Self::Writer {
        self.clone()
    }
}

impl std::io::Write for UniffiLogger {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let message = String::from_utf8_lossy(buf);
        self.0.log(message.to_string());
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[uniffi::export(with_foreign)]
pub trait Logger: Send + Sync {
    fn log(&self, message: String);
}

#[uniffi::export]
pub fn register_logger(logger: Arc<dyn Logger>) {
    // Try to initialize the subscriber, but don't panic if it's already initialized
    let result = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_writer(UniffiLogger(logger))
        .try_init();
    
    match result {
        Ok(_) => eprintln!("Tracing subscriber initialized successfully"),
        Err(e) => eprintln!("Tracing subscriber initialization failed: {:?}", e),
    }
}