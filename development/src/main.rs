use crate::docker::Docker;
use common::database::create_pool;
use common::PodelError;
use notify::{Error, RecursiveMode, Watcher};
use std::fs;
use std::path::Path;
use tokio::process;

pub mod docker;

#[tokio::main]
async fn main() -> Result<(), PodelError> {
    let postgres = Docker::new("podela_me_dev_postgres");
    postgres.start().unwrap();

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

    let _ = tokio::join!(website_task, scss_task, setup_dev());
    Ok(())
}

async fn setup_dev() -> Result<(), PodelError> {
    let admin_user = common::database::user::User::new("admin", "test@example.com", "admin", true)?;
    let pool = create_pool().await?;
    admin_user.register(&pool).await?;
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
