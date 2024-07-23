use anyhow::Result;
use tracing::Subscriber;
use tracing_subscriber::prelude::*;
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::Layer;

pub type Logger<S> = Box<dyn 'static + Layer<S> + Send + Sync>;

pub fn stderr<S>() -> Logger<S>
where
    S: Subscriber,
    S: for<'a> LookupSpan<'a>,
{
    tracing_subscriber::fmt::layer()
        .compact()
        .without_time()
        .with_ansi(false)
        .with_level(false)
        .with_target(false)
        .with_writer(std::io::stderr)
        .boxed()
}

pub fn init() -> Result<()> {
    let logger = stderr();
    let subscriber = tracing_subscriber::registry().with(logger);
    tracing::subscriber::set_global_default(subscriber)?;

    Ok(())
}
