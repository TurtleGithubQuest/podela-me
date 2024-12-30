use clap::Parser;
use std::process::Command;
use thiserror::Error;

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

    pub fn start(&self) -> Result<(), DockerError> {
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

        Ok(())
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
