use tracing::Level;

use examples_common::logging::LogLevelFilter;

fn main() {
    // setup logging
    examples_common::logging::init_logging(LogLevelFilter::builder()
        .global(Level::WARN)
        // set the logging for this executable
        .level(env!("CARGO_CRATE_NAME"), Level::TRACE)
        // set the logging for the examples_common crate
        .level(examples_common::CRATE_NAME, Level::TRACE)
        .build()
    );

    // test the crate
    examples_common::logging::self_log_test();

    // test here
    examples_common::logging::log_level_test!();
}