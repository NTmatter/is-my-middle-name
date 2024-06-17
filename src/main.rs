use lambda_http::{http::Uri, run, service_fn, tracing, Body, Error, Request, Response};

enum Action {
    Redirect(Uri),
    Html(&'static str),
    Unknown(Option<&'static str>),
}

/// Match hostnames and uris, returning redirects or body content.
async fn function_handler(event: Request) -> Result<Response<Body>, Error> {
    let uri = event.uri();

    // Host matches
    let response = match uri.host() {
        Some("localhost") => Action::Html("Local Development Server"),
        Some("lambda.is-my-middle.name") => {
            Action::Html("<strong>TODO</strong>: Placeholder for the actual homepage.")
        }
        Some("check-known.is-my-middle.name") => Action::Redirect(Uri::from_static(
            "https://lambda.is-my-middle.name?utm-campaign=unit-test",
        )),
        _ => Action::Unknown(Some("Unknown URL or Hostname")),
    };

    let builder = Response::builder().header("content-type", "text/html");
    let builder = match response {
        Action::Redirect(uri) => builder
            .status(301)
            .header("Location", uri.to_string())
            .body(
                format!(
                    r#"<head><meta http-equiv="Refresh" content="0; URL={}" /></head>"#,
                    uri
                )
                .into(),
            ),
        Action::Html(body) => builder.status(200).body(body.into()),
        Action::Unknown(Some(message)) => builder.status(404).body(message.into()),
        Action::Unknown(None) => builder.status(404).body("Unknown host or uri".into()),
    };

    let resp = builder.map_err(Box::new)?;

    Ok(resp)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing::init_default_subscriber();

    run(service_fn(function_handler)).await
}
