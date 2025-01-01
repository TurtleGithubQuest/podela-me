use crate::extend_with_app_state;
use common::database::reviewable::website::Website;
use rinja_axum::axum::{
    extract::Path,
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use rinja_axum::*;

extend_with_app_state! {
    #[template(path = "subject/website.html")]
    struct WebsiteSubjectTemplate {
        subject: Option<Website>,
    };
}

pub fn router() -> Router<()> {
    Router::new()
        .route("/website/:id", get(get::website))
}

mod get {
    use super::*;
    use common::AuthSession;

    pub(crate) async fn website(
        auth_session: AuthSession,
        Path(id): Path<String>,
    ) -> Response {
        let subject = Website::find(&auth_session.backend.pool, id).await;
        let template = WebsiteSubjectTemplate::from_app_state(&auth_session, subject.ok());
        rinja_axum::into_response(&template)
    }

}