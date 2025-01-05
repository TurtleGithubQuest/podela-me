use crate::page::{index, partials, subject, user};
use common::database::{create_pool, migrate};
use common::{AppState, PodelError};
use poem::{
    get, handler,
    listener::TcpListener,
    middleware::AddData,
    web::{Data, Path},
    EndpointExt, Route, Server,
};
use fluent_templates::LanguageIdentifier;
use lazy_static::lazy_static;
use std::str::FromStr;
use std::sync::Arc;
use log::error;
use poem::endpoint::StaticFilesEndpoint;
use poem::middleware::Csrf;
use poem::session::{CookieConfig, CookieSession};
use poem::web::cookie::SameSite;
use poem::web::Html;
use rinja::Template;
use tokio::signal;
use tokio::task::AbortHandle;

pub mod page;
pub mod filters;

pub type PoemResult = poem::Result<Html<String>, poem::error::NotFoundError>;

lazy_static! {
    static ref DEFAULT_LANGUAGE: LanguageIdentifier =
        LanguageIdentifier::from_str("en-US").expect("??");
}

#[tokio::main]
async fn main() -> Result<(), PodelError> {
    common::load_config().await;
    let pool = create_pool().await?;

    migrate(&pool).await.unwrap();

    let state =  Arc::new(AppState::new(pool));

    let app = Route::new()
        .nest("/assets/", StaticFilesEndpoint::new("./assets"))
        .nest("/partials", partials::route())
        .nest("/user", user::route_user())
        .nest("/auth", user::route_auth())
        .nest("/website", subject::route_website())
        .at("/", get(index::get))
        .with(CookieSession::new(
            CookieConfig::new()
                .name("cookie")
                .same_site(SameSite::Strict)
                .secure(false)
        ))
        .with(Csrf::new())
        .with(AddData::new(state));

    Server::new(TcpListener::bind("127.0.0.1:3000")).run(app).await?;

    Ok(())
}

#[macro_export]
macro_rules! extend_with_app_state {
    (
        $(
            $(#[$struct_meta:meta])*
            $vis:vis struct $name:ident $(<$life:lifetime>)? {
                $(
                    $(#[$field_meta:meta])*
                    $field_vis:vis $field_name:ident: $field_type:ty
                ),*
                $(,)?
            };
        )*
    ) => {
        #[allow(unused_imports)]
        use crate::DEFAULT_LANGUAGE;
        use fluent_templates::LanguageIdentifier;
        use std::str::FromStr;
        use rinja::Template;
        use std::sync::Arc;
        use poem::{web::{Data}, session::Session};
        use common::AppState;
        use crate::filters as filters;

        $(
            #[allow(dead_code)]
            #[derive(Template)]
            $(#[$struct_meta])*
            $vis struct $name<'a> {
                $(
                    $(#[$field_meta])*
                    $field_vis $field_name: $field_type,
                )*
                pub title: &'a str,
                pub visitors: u64,
                pub user: Option<common::database::user::User>,
                pub user_language: LanguageIdentifier,
            }

            impl<'a> $name<'a> {
                pub fn from_app_state(
                    state: Data<&'a Arc<AppState>>,
                    session: &'a Session,
                    $( $field_name: $field_type, )*
                ) -> Self {
                    Self {
                        $( $field_name, )*
                        title: state.title,
                        visitors: state.visitors,
                        user_language: session
                            .get::<String>("user_language")
                            .and_then(|lang| LanguageIdentifier::from_str(lang.as_str()).ok())
                            .unwrap_or_else(|| DEFAULT_LANGUAGE.clone()),
                        user: session.get::<common::database::user::User>("user"),
                    }
                }
            }
        )*
    };
}

async fn shutdown_signal(deletion_task_abort_handle: AbortHandle) {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => { deletion_task_abort_handle.abort() },
        _ = terminate => { deletion_task_abort_handle.abort() },
    }
}

fn render<T>(template: &T) -> PoemResult
where
    T: Template {
    match template.render() {
        Ok(rendered) => Ok(Html(rendered)),
        Err(e) => {
            error!("Template rendering error: {}", e);
            Err(poem::error::NotFoundError)
        }
    }
}