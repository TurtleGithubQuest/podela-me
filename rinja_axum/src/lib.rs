pub use axum_login::axum;
use axum_login::axum::response::{IntoResponse, Response};
pub use rinja::*;

/// Render a [`Template`] into a [`Response`], or render an error page.
#[must_use]
pub fn into_response<T: ?Sized + rinja::Template>(tmpl: &T) -> Response {
    try_into_response(tmpl)
        .map_err(|err| axum_login::axum::response::ErrorResponse::from(err.to_string()))
        .into_response()
}

/// Try to render a [`Template`] into a [`Response`].
pub fn try_into_response<T: ?Sized + rinja::Template>(tmpl: &T) -> Result<Response, Error> {
    let value = tmpl.render()?.into();
    Response::builder()
        .header(
            http::header::CONTENT_TYPE,
            http::header::HeaderValue::from_static("text/html"),
        )
        .body(value)
        .map_err(|err| Error::Custom(err.into()))
}
