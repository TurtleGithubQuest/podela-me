use crate::docker::Docker;
use common::database::reviewable::website::Website;
use common::database::reviewable::{LegalForm, Organization};
use common::database::{create_pool, migrate};
use common::PodelError;
use notify::{Error, RecursiveMode, Watcher};
use sqlx::{Pool, Postgres};
use std::fs;
use std::path::Path;
use std::sync::Arc;
use tokio::process;
use common::database::comment::Comment;

pub mod docker;

#[tokio::main]
async fn main() -> Result<(), PodelError> {
    let postgres = Docker::new("podela_me_dev_postgres");
    postgres.start().await.map_err(|err| PodelError::DatabaseError(err.to_string()))?;
    let pool = create_pool().await?;
    migrate(&pool).await.expect("Database migration failed");
    setup_dev(&pool).await?;

    let website_task = async {
        let mut website_process = process::Command::new("cargo")
            .arg("watch")
            .arg("-x")
            .arg("run --package website")
            .current_dir(Path::new("website"))
            .spawn()
            .unwrap();

        website_process.wait().await.unwrap();
        postgres.remove().unwrap();
        postgres.stop().unwrap();
    };

    let scss_task = watch_scss();

    let _ = tokio::join!(website_task, scss_task);
    Ok(())
}

async fn setup_dev(pool: &Pool<Postgres>) -> Result<(), PodelError> {
    let admin = common::database::user::User::register(
        &pool,
        "admin",
        Some("test@example.com"),
        "admin",
        true,
    ).await.unwrap_or(common::database::user::User::find("admin", &pool).await?);
    let org = Organization::new("Test org", LegalForm::Sro, Some(admin.clone()));
    let web_test1 = Website::new("test1", "example.com", None::<String>, Some(org));
    let _ = web_test1.save(&pool).await;
    let _ = Website::new("test2", "google.com", Some("Short description test\nyes"), None).save(&pool).await;

    let _ = Comment::new("website", web_test1.id, "test text", Arc::new(admin)).save(&pool).await?;

    Ok(())
}

async fn watch_scss() -> Result<(), Error> {
    let website_dir = Path::new("website");
    let styles_dir = website_dir.join("styles");
    let assets_dir = website_dir.join("assets");

    fs::create_dir_all(&assets_dir).expect("Failed to create assets directory");

    let (tx, rx) = std::sync::mpsc::channel();

    let mut watcher = notify::recommended_watcher(move |res| {
        if let Ok(event) = res {
            tx.send(event).unwrap();
        }
    })?;

    watcher.watch(&styles_dir, RecursiveMode::Recursive)?;
    compile_scss();

    for _ in rx {
        compile_scss();
    }

    Ok(())
}

fn compile_scss() {
    let website_dir = Path::new("website");
    let scss_file = website_dir.join("styles/main.scss");
    let css_file = website_dir.join("assets/main.css");

    match grass::from_path(scss_file, &grass::Options::default()) {
        Ok(css) => {
            if let Err(err) = fs::write(css_file, css) {
                eprintln!("Error writing CSS file: {}", err);
            }
        }
        Err(err) => eprintln!("Error compiling SCSS: {}", err),
    }
}
