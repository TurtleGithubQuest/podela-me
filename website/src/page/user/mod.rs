use crate::extend_with_app_state;
use poem::{get, post, Route, handler, web::Path};
use common::database::user::User;

extend_with_app_state! {
    #[template(path = "user/profile.html")]
    struct UserProfileTemplate {
        profile: Option<User>,
    };

    #[template(path = "user/auth.html")]
    struct UserAuthTemplate {};
}

pub fn route_user() -> Route {
    Route::new()
        .at("/:id", get(get::profile))
}

pub fn route_auth() -> Route {
    Route::new()
        .at("/", get(get::auth).post(post::auth))
        .at("/logout", post(post::logout))
}

mod get {
    use super::*;
    use crate::PoemResult;

    #[handler]
    pub(crate) async fn profile(
        state: Data<&Arc<AppState>>,
        session: &Session,
        Path(user_id): Path<String>,
    ) -> PoemResult {
        let profile = User::find(user_id, &state.pool).await;
        let template = UserProfileTemplate::from_app_state(state, session, profile.ok());
        crate::render(&template)
    }

    #[handler]
    pub(crate) async fn auth(
        state: Data<&Arc<AppState>>,
        session: &Session,
    ) -> PoemResult {
        let template = UserAuthTemplate::from_app_state(state, session);
        crate::render(&template)
    }
}

mod post {
    use log::error;
    use super::*;
    use poem::{IntoResponse, Response};
    use poem::http::StatusCode;
    use poem::web::Form;
    use common::database::user::{is_valid, verify_password, Credentials, SessionData};
    use crate::PoemResult;

    #[handler]
    pub(crate) async fn logout(
        session: &Session
    ) -> impl IntoResponse {
        session.clear();
        Response::builder().status(StatusCode::OK).finish()
    }

    #[handler]
    pub(crate) async fn auth(
        state: Data<&Arc<AppState>>,
        session: &Session,
        Form(creds): Form<Credentials>,
    ) -> StatusCode {
        let status = if creds.authentication {
            login(state, session, creds).await
        } else {
            register(state, session, creds).await
        };
        status
    }

    async fn register(
        state: Data<&Arc<AppState>>,
        session: &Session,
        creds: Credentials) -> StatusCode {
        if creds.username.is_empty() || creds.password.is_empty() {
            return StatusCode::BAD_REQUEST;
        }

        if User::find(&creds.username, &state.pool).await.is_ok() {
            return StatusCode::CONFLICT;
        }

        match User::register(
            &state.pool,
            creds.username,
            None::<String>,
            creds.password,
            false,
        )
        .await {
            Ok(user) => {
                if let Err(err) = user.create_session(&state.pool, session, None::<String>).await { // TODO: fetch ip
                   error!("{}", err);
                }
                StatusCode::OK
            }
            Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    async fn login(
        state: Data<&Arc<AppState>>,
        session: &Session,
        creds: Credentials
    ) -> StatusCode {
        let user = match User::find(&creds.username, &state.pool).await {
            Ok(user) => user,
            Err(_) => return StatusCode::UNAUTHORIZED,
        };

        if is_valid(&session) {
            return StatusCode::CONFLICT;
        }

        if verify_password(creds.password, &user.password_hash).is_err() {
            return StatusCode::UNAUTHORIZED;
        }

        if let Err(err) = user.create_session(&state.pool, session, None::<String>).await { // TODO: fetch ip
           error!("{}", err);
        }

        StatusCode::OK
    }
}
