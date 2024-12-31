use crate::extend_with_app_state;
use axum_macros::debug_handler;
use common::AuthSession;
use rinja_axum::axum::response::IntoResponse;
use rinja_axum::Template;

extend_with_app_state! {
    #[template(path = "index.html")]
    struct IndexTemplate<'a> {
        name: &'a str,
    }
}

#[debug_handler]
pub(crate) async fn get(auth_session: AuthSession) -> impl IntoResponse {
    let template = IndexTemplate::from_app_state(&auth_session, "test");
    rinja_axum::into_response(&template)
}
