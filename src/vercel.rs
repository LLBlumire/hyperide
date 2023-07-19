use std::ops::Deref;

use axum::{body::Body as AxumBody, body::HttpBody, http::Request as HttpRequest, Router};
use http::Uri;
use tower_service::Service;
use url::Url;
use vercel_runtime::{run as vercel_run, Body, Error, Request, Response};

/// Runs an axum router in the vercel runtime, responding to a request
pub async fn run(app: Router) -> Result<(), Error> {
    let handler = |mut req: Request| {
        let mut app = app.clone();
        async move {
            // Get supplied path from vercel rewrite
            let url = Url::parse(&req.uri().to_string()).unwrap();
            let path = url
                .query_pairs()
                .find_map(|(k, v)| (k == "path").then_some(v))
                .unwrap();

            // Compute original path based on path uri
            let new_path = format!("/{}?{}", path, req.uri().query().unwrap_or_default())
                .parse()
                .unwrap();

            // Alter request URI to match
            let mut uri = req.uri().clone().into_parts();
            let path_and_query = uri.path_and_query.as_mut().unwrap();
            *path_and_query = new_path;
            *req.uri_mut() = Uri::from_parts(uri).unwrap();

            // Convert into Axum/Http Request
            let (req_parts, req_body) = req.into_parts();
            let req_body = req_body.deref().to_owned();
            let req = HttpRequest::from_parts(req_parts, AxumBody::from(req_body));

            // Call the axum router
            let initial_response = app.call(req).await?;

            // Convert back into aws lambda response
            let (parts, mut initial_body) = initial_response.into_parts();
            let mut buffer = Vec::new();
            while let Some(bytes) = initial_body.data().await {
                let bytes = bytes?;
                buffer.extend(bytes);
            }
            let final_response = Response::from_parts(parts, Body::Binary(buffer));

            Ok(final_response)
        }
    };
    vercel_run(handler).await
}
