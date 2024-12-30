use crate::docker::Docker;
use tokio::process;

pub mod docker;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let postgres = Docker::new("podela_me_dev_postgres");
    postgres.start()?;

    let mut website_process = process::Command::new("cargo")
        .arg("run")
        .arg("--package")
        .arg("website")
        .spawn()
        .unwrap();

    website_process.wait().await.unwrap();
    postgres.remove()?;
    postgres.stop()?;
    Ok(())
}
