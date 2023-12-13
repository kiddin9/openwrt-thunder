use axum::body::Bytes;
use axum::response::{IntoResponse, Response};
use axum::{
    async_trait,
    extract::FromRequest,
    http::{self, Request},
};
use http::header::CONTENT_TYPE;
use http::Uri;

/// Extractor for request parts.
pub struct RequestExt {
    pub uri: Uri,
    pub method: http::Method,
    pub headers: http::HeaderMap,
    pub body: Option<Bytes>,
}

#[async_trait]
impl<S, B> FromRequest<S, B> for RequestExt
where
    Bytes: FromRequest<S, B>,
    B: Send + 'static,
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request(req: Request<B>, state: &S) -> Result<Self, Self::Rejection> {
        let (parts, body) = req.into_parts();

        let body = if parts.headers.get(CONTENT_TYPE).is_some() {
            Some(
                Bytes::from_request(Request::new(body), state)
                    .await
                    .map_err(IntoResponse::into_response)?,
            )
        } else {
            None
        };

        Ok(RequestExt {
            uri: parts.uri,
            method: parts.method,
            headers: parts.headers,
            body,
        })
    }
}
