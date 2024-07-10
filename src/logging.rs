pub fn setup_tracing() {
    #[cfg(not(target_arch = "wasm32"))]
    {
        tracing_subscriber::fmt::init();
    }
    #[cfg(target_arch = "wasm32")]
    {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        tracing_subscriber::fmt()
            .with_writer(
                // To avoide trace events in the browser from showing their
                // JS backtrace, which is very annoying, in my opinion
                tracing_subscriber_wasm::MakeConsoleWriter::default()
                    .map_trace_level_to(tracing::Level::DEBUG),
            )
            // For some reason, if we don't do this in the browser, we get
            // a runtime error.
            .without_time()
            .init();
    }

    // test all levels of logging
    tracing::trace!("This is a trace message");
    tracing::debug!("This is a debug message");
    tracing::info!("This is an info message");
    tracing::warn!("This is a warning message");
    tracing::error!("This is an error message");
}
