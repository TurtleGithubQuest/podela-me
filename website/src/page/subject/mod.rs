use crate::extend_with_app_state;
use common::database::reviewable::website::Website;
use poem::{get, Route, handler, web::Path};

extend_with_app_state! {
    #[template(path = "subject/website.html")]
    struct WebsiteSubjectTemplate {
        subject: Option<Website>,
    };

    #[template(path = "subject/website/list.html")]
    struct WebsiteListTemplate {
        subjects: Vec<Website>
    };
}

pub fn route_website() -> Route {
    Route::new()
        .at("/", get(get::list))
        .at("/:id", get(get::website))
}

mod get {
    use log::error;
    use super::*;
    use crate::PoemResult;

    #[handler]
    pub(crate) async fn list(
        state: Data<&Arc<AppState>>,
        session: &Session,
    ) -> PoemResult {
        let subjects =
            Website::find_multiple(&state.pool, 10_i64, 0_i64).await.unwrap_or_else(|err| {
                error!("{:?}", err);
                Vec::new()
            });
        let template = WebsiteListTemplate::from_app_state(state, session, subjects);

        crate::render(&template)
    }

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