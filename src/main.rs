use clap::Parser;
use rorm_cli::cli::Cli;
use tracing::Level;
use tracing_subscriber::filter::filter_fn;
use tracing_subscriber::fmt;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let cli: Cli = Cli::parse();

    // Basically converts `info!` into `println!`
    tracing_subscriber::registry()
        .with(fmt::layer().compact().without_time().with_target(false))
        .with(filter_fn(|metadata| {
            metadata.is_event()
                && *metadata.level() <= Level::INFO
                && metadata.target().starts_with("rorm_cli")
        }))
        .init();

    cli.run().await
}
