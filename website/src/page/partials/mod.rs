use crate::extend_with_app_state;
use rinja_axum::axum::{
    response::IntoResponse,
    routing::get,
    Router
};

extend_with_app_state! {
    #[template(path = "navbar.html")]
    struct NavbarTemplate {};
}

pub fn router() -> Router<()> {
    Router::new()
        .route("/partials/navbar", get(get::navbar))
}

mod get {
    use super::*;
    use common::AuthSession;

    pub(crate) async fn navbar(auth_session: AuthSession,) -> impl IntoResponse {
        let template = NavbarTemplate::from_app_state(&auth_session);
        rinja_axum::into_response(&template)
    }
}