use poem::{get, Route, handler};
use crate::{extend_with_app_state, PoemResult};
use common::{database::user::User};

extend_with_app_state! {
    #[template(path = "partials/navbar.html")]
    struct NavbarTemplate {};

    #[template(path = "partials/modals/user/profile.html")]
    struct UserProfileModalTemplate {
        id: String,
        profile: Option<User>,
    };
}

pub fn route() -> Route {
    Route::new()
        .at("/navbar", get(get::navbar))
        .at("/modals/user/profile/:id", get(get::modals::profile))
}

mod get {
    use super::*;

    #[handler]
    pub(crate) async fn navbar(state: Data<&Arc<AppState>>, session: &Session) -> PoemResult {
        let template = NavbarTemplate::from_app_state(state, session);
        crate::render(&template)
    }

    pub(crate) mod modals {
        use super::*;
        use poem::web::Path;

        #[handler]
        pub(crate) async fn profile(
            state: Data<&Arc<AppState>>,
            session: &Session,
            Path(id): Path<String>,
        ) -> PoemResult {
            let profile = User::find(id, &state.pool).await;
            let modal_id = format!("user-link-modal-{}", profile.as_ref().map(|user| user.id.as_str()).unwrap_or("not_found"));

            let template = UserProfileModalTemplate::from_app_state(state, session, modal_id, profile.ok());
            crate::render(&template)
        }

    }

}