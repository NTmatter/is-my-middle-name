// SPDX-License-Identifier: MIT

use lambda_http::{Body, Error, http::Uri, Request, Response, run, service_fn, tracing};
use lambda_http::http::StatusCode;

/// Allow a small set of internal redirects.
/// TODO Consider increasing the MAX_COUNT as we are traversing and coalescing our internal graph.
const MAX_REDIRECTS: usize = 5;

enum Action {
    Redirect(Uri),
    Html(String),
    Unknown(Option<String>),
    Error(StatusCode, String),
}

/// Choose a redirect, static content, or Unknown
fn choose_action(uri: &Uri) -> Action {
    let Some(hostname) = uri.host() else {
        return Action::Error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "No hostname was specified".to_string(),
        );
    };

    let action = match hostname {
        "localhost" => Action::Html("Local Development Server".to_string()),
        "lambda.is-my-middle.name" => {
            Action::Html("<strong>TODO</strong>: Placeholder for the actual homepage.".to_string())
        }
        // TODO Detect port from current request and change hostname.
        "check-known.is-my-middle.name" => Action::Redirect(Uri::from_static(
            "https://lambda.is-my-middle.name?utm-campaign=unit-test",
        )),
        _ => Action::Unknown(Some("Unknown URL or Hostname".to_string())),
    };

    // Detect immediate recursion.
    if let Action::Redirect(redirect_uri) = &action {
        if redirect_uri
            .host()
            .is_some_and(|redirect_host| redirect_host.eq(hostname))
        {
            return Action::Error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Redirects to self. Aborting early to avoid recursion.".to_string(),
            );
        }
    };

    action
}

/// Ensure that there are no redirect loops for the chosen input.
/// Small loops up to `MAX_REDIRECTS` will be resolved to their final destination.
fn consume_cycles(uri: &Uri) -> Result<Action, String> {
    let mut action = choose_action(uri);
    for _ in 0..=MAX_REDIRECTS {
        if let Action::Redirect(next_uri) = &action {
            let next_action = choose_action(next_uri);
            if matches!(next_action, Action::Redirect(_)) {
                action = next_action
            } else {
                return Ok(action);
            }
        } else {
            return Ok(action);
        }
    }

    Err("Too many internal redirects. Aborting early to avoid recursion.".to_string())
}

/// Match hostnames and uris, returning redirects or body content.
async fn function_handler(event: Request) -> Result<Response<Body>, Error> {
    let uri = event.uri();

    // Choose an action based on URL
    let response = consume_cycles(uri)
        .unwrap_or_else(|err| Action::Error(StatusCode::INTERNAL_SERVER_ERROR, err));

    let builder = Response::builder().header("content-type", "text/html");
    let builder = match response {
        Action::Redirect(uri) => builder
            .status(StatusCode::PERMANENT_REDIRECT)
            .header("Location", uri.to_string())
            .body(
                format!(
                    r#"<head><meta http-equiv="Refresh" content="0; URL={}" /></head>"#,
                    uri
                )
                .into(),
            ),
        Action::Html(body) => builder.status(StatusCode::OK).body(body.into()),
        Action::Unknown(Some(message)) => {
            builder.status(StatusCode::NOT_FOUND).body(message.into())
        }
        Action::Unknown(None) => builder
            .status(StatusCode::NOT_FOUND)
            .body("Unknown host or uri".into()),
        Action::Error(code, msg) => builder.status(code).body(msg.into()),
    };

    let resp = builder.map_err(Box::new)?;

    Ok(resp)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing::init_default_subscriber();

    run(service_fn(function_handler)).await
}
