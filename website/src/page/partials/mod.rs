use crate::extend_with_app_state;
use common::database::user::User;
use rinja_axum::axum::{
    response::IntoResponse,
    routing::get,
    Router
};

extend_with_app_state! {
    #[template(path = "partials/navbar.html")]
    struct NavbarTemplate {};

    #[template(path = "partials/modals/user/profile.html")]
    struct UserProfileModalTemplate {
        id: String,
        profile: Option<User>,
    };
}

pub fn router() -> Router<()> {
    Router::new()
        .route("/partials/navbar", get(get::navbar))
        .route("/partials/modals/user/profile/:id", get(get::modals::profile))
}

mod get {
    use super::*;
    use common::AuthSession;

    pub(crate) async fn navbar(auth_session: AuthSession,) -> impl IntoResponse {
        let template = NavbarTemplate::from_app_state(&auth_session);
        rinja_axum::into_response(&template)
    }

    pub(crate) mod modals {
        use super::*;
        use axum::extract::Path;

        pub(crate) async fn profile(
            auth_session: AuthSession,
            Path(id): Path<String>,
        ) -> impl IntoResponse {
            let profile = User::find(id, &auth_session.backend.pool).await;
            let modal_id = format!("user-link-modal-{}", profile.as_ref().map(|user| user.id.as_str()).unwrap_or("not_found"));
            let template = UserProfileModalTemplate::from_app_state(&auth_session, modal_id, profile.ok());
            rinja_axum::into_response(&template)
        }

    }

}