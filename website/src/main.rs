use crate::page::{index, partials, user};
use axum_login::axum::{
    extract::{FromRef, FromRequestParts},
    http::StatusCode,
    routing::get,
    {Router, ServiceExt},
};
use axum_login::tower_sessions::cookie::time::Duration;
use axum_login::tower_sessions::{Expiry, SessionManagerLayer};
use axum_login::AuthManagerLayerBuilder;
use common::database::{create_pool, migrate};
use common::{AppState, PodelError};
use fluent_templates::LanguageIdentifier;
use lazy_static::lazy_static;
use std::str::FromStr;
use tokio::net::TcpListener;
use tokio::signal;
use tokio::task::AbortHandle;
use tower_http::services::ServeDir;
use tower_sessions::cookie::Key;
use tower_sessions::ExpiredDeletion;
use tower_sessions_sqlx_store::PostgresStore;

pub mod page;

lazy_static! {
    static ref DEFAULT_LANGUAGE: LanguageIdentifier =
        LanguageIdentifier::from_str("en-US").expect("??");
}

#[tokio::main]
async fn main() -> Result<(), PodelError> {
    common::load_config().await;
    let pool = create_pool().await?;

    migrate(&pool).await.unwrap();

    let session_store = PostgresStore::new(pool.clone());
    session_store.migrate().await?;

    let deletion_task = tokio::task::spawn(
        session_store
            .clone()
            .continuously_delete_expired(tokio::time::Duration::from_secs(60)),
    );

    let key = Key::generate();

    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false)
        .with_expiry(Expiry::OnInactivity(Duration::days(1)))
        .with_signed(key);

    let state = AppState::new(pool);
    let auth_layer = AuthManagerLayerBuilder::new(state, session_layer).build();

    let app = Router::new()
        .route("/", get(index::get))
        .merge(user::router())
        .merge(partials::router())
        .nest_service("/assets", ServeDir::new("assets"))
        .layer(auth_layer);

    let listener = TcpListener::bind("127.0.0.1:3000").await?;

    axum::serve(listener, app.into_make_service())
        .with_graceful_shutdown(shutdown_signal(deletion_task.abort_handle()))
        .await?;

    deletion_task.await?.unwrap();

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
        use crate::DEFAULT_LANGUAGE;
        use fluent_templates::LanguageIdentifier;
        use std::str::FromStr;
        use rinja_axum::Template;
        use rinja_axum::filters as filters;

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
                    auth_session: &'a common::AuthSession, $( $field_name: $field_type, )*
                ) -> Self {
                    Self {
                        $( $field_name, )*
                        title: auth_session.backend.title,
                        visitors: auth_session.backend.visitors,
                        user_language: auth_session
                            .user
                            .as_ref()
                            .and_then(|user| LanguageIdentifier::from_str(&user.language).ok())
                            .unwrap_or_else(|| DEFAULT_LANGUAGE.clone()),
                        user: auth_session.user.clone(),
                    }
                }
            }
        )*
    };
}

/// Utility function for mapping any error into a `500 Internal Server Error` response.
fn internal_error<E>(err: E) -> (StatusCode, String)
where
    E: std::error::Error,
{
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
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
