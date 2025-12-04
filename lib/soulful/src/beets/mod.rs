use std::{
    io::{Error, Result},
    path::Path,
};
use tokio::process::Command;
use tracing::info;

pub async fn import(sources: Vec<String>, target: &Path) -> Result<()> {
    let config_path =
        std::env::var("BEETS_CONFIG").unwrap_or_else(|_| "beets_config.yaml".to_string());

    info!(
        "Starting beet import for {} items to {:?} using config {}",
        sources.len(),
        target,
        config_path
    );

    let mut cmd = Command::new("beet");
    cmd.arg("-c")
        .arg(&config_path)
        .arg("import")
        .arg("-q") // quiet mode: do not ask for confirmation
        .arg("-d") // destination directory
        .arg(target);

    for source in sources {
        cmd.arg(source);
    }

    // If SINGLETON_MODE env var is set, use -s
    if std::env::var("BEETS_SINGLETON").is_ok() {
        cmd.arg("-s");
    }

    let status = cmd.status().await?;

    if status.success() {
        info!("Beet import successful");
        Ok(())
    } else {
        Err(Error::other("Beet import failed"))
    }
}
