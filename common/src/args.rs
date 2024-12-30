use crate::database::DbCredentials;
use clap::Parser;

#[derive(Parser)]
pub struct CliArgs {
    #[clap(flatten)]
    pub db: DbCredentials,
}
