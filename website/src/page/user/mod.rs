use crate::extend_with_app_state;
use axum_login::axum::{
    extract::Path,
    routing::{get, post},
    Router,
};
use common::database::user::User;
use common::AuthSession;
use rinja_axum::axum::response::{IntoResponse, Response};
use rinja_axum::Template;
use rinja_axum::*;

extend_with_app_state! {
    #[template(path = "user/profile.html")]
    struct UserProfileTemplate {
        profile: Option<User>,
    }

    #[template(path = "user/auth.html")]
    struct UserAuthTemplate {}
}

pub fn router() -> Router<()> {
    Router::new()
        .route("/user/:id", get(get::profile))
        .route("/auth", get(get::auth))
        .route("/auth", post(post::auth))
}

mod get {
    use super::*;
    use axum_macros::debug_handler;
    use common::AuthSession;

    #[debug_handler]
    pub(crate) async fn profile(
        auth_session: AuthSession,
        Path(user_id): Path<String>,
    ) -> Response {
        let profile = User::find(user_id, &auth_session.backend.pool).await;
        let template = UserProfileTemplate::from_app_state(&auth_session, profile.ok());
        rinja_axum::into_response(&template)
    }

    #[debug_handler]
    pub(crate) async fn auth(mut auth_session: AuthSession) -> Response {
        let template = UserAuthTemplate::from_app_state(&auth_session);
        rinja_axum::into_response(&template)
    }
}

mod post {
    use super::*;
    use axum::http::StatusCode;
    use axum::Form;
    use common::database::user::Credentials;

    pub(crate) async fn auth(
        mut auth_session: AuthSession,
        Form(creds): Form<Credentials>,
    ) -> impl IntoResponse {
        if creds.authentication {
            login(auth_session, creds).await
        } else {
            register(auth_session, creds).await
        }
        .into_response()
    }

    async fn register(mut auth_session: AuthSession, creds: Credentials) -> StatusCode {
        StatusCode::OK
    }

    async fn login(mut auth_session: AuthSession, creds: Credentials) -> StatusCode {
        let user = match auth_session.authenticate(creds.clone()).await {
            Ok(Some(user)) => user,
            Ok(None) => {
                return StatusCode::UNAUTHORIZED;
            }
            Err(_) => return StatusCode::INTERNAL_SERVER_ERROR,
        };

        if auth_session.login(&user).await.is_err() {
            StatusCode::INTERNAL_SERVER_ERROR
        } else {
            StatusCode::OK
        }
    }
}
