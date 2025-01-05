use crate::extend_with_app_state;
use common::database::reviewable::website::Website;
use poem::{get, Route, handler, web::Path};

extend_with_app_state! {
    #[template(path = "subject/website.html")]
    struct WebsiteSubjectTemplate {
        subject: Option<Website>,
    };
}

pub fn route_website() -> Route {
    Route::new()
        .at("/:id", get(get::website))
}

mod get {
    use super::*;
    use crate::PoemResult;

    #[handler]
    pub(crate) async fn website(
        state: Data<&Arc<AppState>>,
        session: &Session,
        Path(id): Path<String>,
    ) -> PoemResult {
        let subject = Website::find(&state.pool, id).await;
        let template = WebsiteSubjectTemplate::from_app_state(state, session, subject.ok());

        crate::render(&template)
    }

}