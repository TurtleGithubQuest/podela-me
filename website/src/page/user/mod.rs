use crate::page::WebData;
use crate::AppState;
use axum::extract::{Path, State};
use axum_core::response::Response;
use common::database::user::UserView;
use rinja_axum::axum_core::response::IntoResponse;
use rinja_axum::Template;

#[derive(Template)]
#[template(path = "user/profile.html")]
struct UserProfileTemplate<'a> {
    data: WebData<'a>,
    user: Option<UserView>,
}

pub(crate) async fn get_profile(
    Path(user_id): Path<String>,
    State(state): State<AppState>,
) -> Response {
    let template = UserProfileTemplate {
        user: None,
        data: state.data,
    };
    rinja_axum::into_response(&template)
}
