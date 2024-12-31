use clap::Parser;
use std::process::Command;
use std::time::Duration;
use thiserror::Error;
use tokio::time::sleep;

pub struct Docker {
    name: String,
    image: String,
}

#[derive(Error, Debug)]
pub enum DockerError {
    #[error("Docker Desktop is not running. Please start Docker Desktop and try again.")]
    DockerNotRunning,

    #[error("Failed to pull image: {0}")]
    PullError(String),

    #[error("Failed to inspect container: {0}")]
    InspectError(String),

    #[error("Failed to start container: {0}")]
    StartError(String),

    #[error("Failed to stop container: {0}")]
    StopError(String),

    #[error("Failed to remove container: {0}")]
    RemoveError(String),

    #[error(transparent)]
    IoError(#[from] std::io::Error),
}

impl Docker {
    pub fn new(name: impl ToString) -> Self {
        Self {
            name: name.to_string(),
            image: String::from("postgres:15"),
        }
    }

    fn check_docker_running(&self) -> Result<(), DockerError> {
        let output = Command::new("docker").args(["info"]).output()?;

        if !output.status.success() {
            return Err(DockerError::DockerNotRunning);
        }
        Ok(())
    }

    pub fn setup_image(&self) -> Result<(), DockerError> {
        self.check_docker_running()?;

        let output = Command::new("docker")
            .args(["pull", &self.image])
            .output()?;

        if !output.status.success() {
            return Err(DockerError::PullError(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        Ok(())
    }

    fn container_exists(&self) -> Result<bool, DockerError> {
        let output = Command::new("docker")
            .args(["container", "inspect", &self.name])
            .output()?;

        Ok(output.status.success())
    }

    async fn start_existing(&self) -> Result<(), DockerError> {
        let output = Command::new("docker")
            .args(["start", &self.name])
            .output()?;

        if !output.status.success() {
            return Err(DockerError::StartError(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        self.wait_for_postgres().await?;
        Ok(())
    }

    pub async fn start(&self) -> Result<(), DockerError> {
        self.check_docker_running()?;

        if self.container_exists()? {
            return self.start_existing().await;
        }

        self.setup_image()?;
        let db_credentials = common::args::CliArgs::parse().db;
        let output = Command::new("docker")
            .args([
                "run",
                "-d",
                "--name",
                &self.name,
                "-e",
                &format!("POSTGRES_PASSWORD={}", db_credentials.password),
                "-e",
                &format!("POSTGRES_USER={}", db_credentials.username),
                "-e",
                "POSTGRES_DB=development_db",
                "-p",
                "5432:5432",
                "postgres:15",
            ])
            .output()?;

        if !output.status.success() {
            return Err(DockerError::StartError(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        self.wait_for_postgres().await?;
        Ok(())
    }

    async fn wait_for_postgres(&self) -> Result<(), DockerError> {
        let max_attempts = 60;
        let mut attempts = 0;

        while attempts < max_attempts {
            let output = Command::new("docker")
                .args([
                    "exec",
                    &self.name,
                    "pg_isready",
                    "-U",
                    &common::args::CliArgs::parse().db.username,
                ])
                .output()?;

            if output.status.success() {
                return Ok(());
            }

            attempts += 1;
            sleep(Duration::from_millis(100 * (2_u64.pow(attempts)))).await;
        }

        Err(DockerError::StartError("PostgreSQL failed to become ready".into()))
    }

    pub fn stop(&self) -> Result<(), DockerError> {
        self.check_docker_running()?;
        let output = Command::new("docker").args(["stop", &self.name]).output()?;

        if !output.status.success() {
            return Err(DockerError::StopError(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        Ok(())
    }

    pub fn remove(&self) -> Result<(), DockerError> {
        self.check_docker_running()?;
        let output = Command::new("docker")
            .args(["rm", "-f", &self.name])
            .output()?;

        if !output.status.success() {
            return Err(DockerError::RemoveError(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        Ok(())
    }
}
