use clap::Parser;

#[derive(Parser, Debug)]
pub struct DbCredentials {
    #[clap(long, env = "DB_HOST", default_value = "localhost")]
    pub host: String,
    #[clap(long, env = "DB_PORT", default_value = "5432")]
    pub port: u16,
    #[clap(long, env = "DB_USERNAME", default_value = "username")]
    pub username: String,
    #[clap(long, env = "DB_PASSWORD", default_value = "password")]
    pub password: String,
    #[clap(long, env = "DB_NAME", default_value = "development_db")]
    pub name: String,
}
