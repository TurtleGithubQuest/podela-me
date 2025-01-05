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
    use super::*;
    use poem::{IntoResponse, Response};
    use poem::http::StatusCode;
    use poem::web::Form;
    use common::database::user::Credentials;
    use crate::PoemResult;

    #[handler]
    pub(crate) async fn logout(
        session: &Session
    ) -> impl IntoResponse {
        session.purge();
        Response::builder().status(StatusCode::OK).finish()
    }

    #[handler]
    pub(crate) async fn auth(
        state: Data<&Arc<AppState>>,
        session: &Session,
        Form(creds): Form<Credentials>,
    ) -> PoemResult {
        todo!()
        /*if creds.authentication {
            login(auth_session, creds).await
        } else {
            register(auth_session, creds).await
        }*/
    }

    async fn register(
        state: Data<&Arc<AppState>>,
        session: &Session,
        creds: Credentials) -> StatusCode {
        todo!()
    }

    async fn login(
        state: Data<&Arc<AppState>>,
        session: &Session,
        creds: Credentials
    ) -> StatusCode {
        todo!()
        /*let user = match auth_session.authenticate(creds.clone()).await {
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
        }*/
    }
}
