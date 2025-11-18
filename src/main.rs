use clap::Parser;
use rorm_cli::cli::Cli;

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let cli: Cli = Cli::parse();
    cli.run().await
}
