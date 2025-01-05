use crate::{extend_with_app_state, PoemResult};
use poem::{handler};

extend_with_app_state! {
    #[template(path = "index.html")]
    struct IndexTemplate {};
}

#[handler]
pub(crate) async fn get(state: Data<&Arc<AppState>>, session: &Session) -> PoemResult {
    let template = IndexTemplate::from_app_state(state, session);
    crate::render(&template)
}
