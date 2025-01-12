use poem::{get, Route, handler};
use crate::{extend_with_app_state, PoemResult};
use common::{database::user::User};
use common::database::comment::{Comment, CommentParent, Commentable};
use serde::Deserialize;

extend_with_app_state! {
    #[template(path = "partials/navbar.html")]
    struct NavbarTemplate {};

    #[template(path = "partials/modals/user/profile.html")]
    struct UserProfileModalTemplate {
        id: String,
        profile: Option<User>,
    };

    #[template(path = "partials/comments.html")]
    struct CommentsTemplate {
        comments: Vec<Comment>,
    };
}

#[derive(Debug, serde::Deserialize)]
pub struct DbQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub fn route() -> Route {
    Route::new()
        .at("/comments/:parent_type/:parent_id", get(get::comments))
        .at("/navbar", get(get::navbar))
        .at("/modals/user/profile/:id", get(get::modals::profile))
}

mod get {
    use log::error;
    use poem::web::{Path, Query};
    use common::database::reviewable::website::Website;
    use super::*;

    #[handler]
    pub(crate) async fn comments(
        state: Data<&Arc<AppState>>,
        session: &Session,
        Path((parent_type, parent_id)): Path<(String, String)>,
        Query(query_params): Query<DbQuery>
    ) -> PoemResult {
        let limit = query_params.limit.unwrap_or(10_i64);
        let offset = query_params.offset.unwrap_or(0_i64);

        let comments = match parent_type.as_str() {
                "website"|"user" => Comment::find_multiple(&state.pool, parent_type, parent_id, limit, offset).await,
                _ => return Err(poem::error::NotFoundError),
            }
            .map_err(|err| {
                error!("Failed to fetch comments: {}", err);
                poem::error::NotFoundError
            })?;
        println!("{:?}", comments);
        let template = CommentsTemplate::from_app_state(state, session, comments);
        crate::render(&template)
    }

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