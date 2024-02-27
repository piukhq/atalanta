use color_eyre::Result;

/// Initializes global state such as logging and error handling.
///
/// # Errors
///
/// Returns an error if the eyre panic handler cannot be installed.
pub fn startup() -> Result<()> {
    color_eyre::install()?;
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_target(false)
        .init();

    Ok(())
}
