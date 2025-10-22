use std::convert::Infallible;

use axum::{
    extract::{FromRef, FromRequestParts},
    http::{request::Parts, HeaderMap},
};

use crate::state::AppState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhoAmI {
    Human,
    Bot,
    Unknown,
}

impl WhoAmI {
    pub fn from_header(header: &HeaderMap) -> Self {
        let sec_fetch_dest = header.get("sec-fetch-dest").and_then(|v| v.to_str().ok());
        let accept = header.get("accept").and_then(|v| v.to_str().ok());
        let sec_fetch_mode = header.get("sec-fetch-mode").and_then(|v| v.to_str().ok());

        match sec_fetch_mode {
            Some("navigate") => return WhoAmI::Human,
            Some("no-cors") => return WhoAmI::Bot,
            _ => {}
        }

        match sec_fetch_dest {
            Some("image") => return WhoAmI::Bot,
            Some("document") => return WhoAmI::Human,
            _ => {}
        }

        match accept {
            Some(accept) if accept.contains("text/html") => return WhoAmI::Human,
            Some(accept) if accept.contains("image/") => return WhoAmI::Bot,
            _ => {}
        }
        WhoAmI::Unknown
    }
}
#[derive(Debug, Clone)]
pub struct ExtractWhoAmI(pub WhoAmI);

impl<S> FromRequestParts<S> for ExtractWhoAmI
where
    S: Send + Sync,
    AppState: FromRef<S>,
{
    type Rejection = Infallible;

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        Ok(ExtractWhoAmI(WhoAmI::from_header(&parts.headers)))
    }
}
