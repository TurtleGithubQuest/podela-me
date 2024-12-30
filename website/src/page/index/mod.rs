use crate::page::WebData;
use crate::AppState;
use axum::extract::State;
use axum_core::response::Response;
use axum_macros::debug_handler;
use rinja_axum::axum_core::response::IntoResponse;
use rinja_axum::Template;

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate<'a> {
    data: WebData<'a>,
    name: &'a str,
}

#[debug_handler]
pub(crate) async fn get(State(state): State<AppState>) -> Response {
    let template = IndexTemplate {
        name: "world",
        data: state.data,
    };
    rinja_axum::into_response(&template)
}
