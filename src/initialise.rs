use color_eyre::Result;

pub fn startup() -> Result<()> {
    color_eyre::install()?;
    tracing_subscriber::fmt()
    .with_max_level(tracing::Level::DEBUG)
    .with_target(false)
    .init();

    Ok(())
}
