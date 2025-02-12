use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use wave::cli::run_cli;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or(EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(env_filter)
        .init();

    run_cli().await?;
    Ok(())
}
