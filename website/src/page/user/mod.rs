use crate::page::WebData;
use crate::AppState;
use axum::extract::{Path, State};
use axum_core::response::Response;
use common::database::user::User;
use rinja_axum::axum_core::response::IntoResponse;
use rinja_axum::Template;
use rinja_axum::*;

#[derive(Template)]
#[template(path = "user/profile.html")]
struct UserProfileTemplate<'a> {
    data: WebData<'a>,
    user: Option<User>,
}

pub(crate) async fn get_profile(
    Path(user_id): Path<String>,
    State(state): State<AppState>,
) -> Response {
    let user = User::find(user_id, &state.pool).await;
    let template = UserProfileTemplate {
        user: user.ok(),
        data: state.data,
    };
    rinja_axum::into_response(&template)
}
