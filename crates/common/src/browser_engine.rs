use axum::http::HeaderMap;

pub enum WellKnownBrowserEngine {
    Blink,
    WebKit,
    Gecko,
    Unknown,
}

impl WellKnownBrowserEngine {
    pub fn from_header(headers: &HeaderMap) -> Self {
        if let Some(sec_ch_ua) = headers.get("sec-ch-ua") {
            if let Ok(ua_str) = sec_ch_ua.to_str() {
                let ua_lower = ua_str.to_lowercase();
                if ua_lower.contains("chromium") || ua_lower.contains("chrome") {
                    return WellKnownBrowserEngine::Blink;
                } else if ua_lower.contains("gecko") {
                    return WellKnownBrowserEngine::Gecko;
                } else if ua_lower.contains("safari") {
                    return WellKnownBrowserEngine::WebKit;
                }
            }
        }
        WellKnownBrowserEngine::Unknown
    }

    pub fn is_x_multipart_replace_double_frame(&self) -> bool {
        matches!(self, WellKnownBrowserEngine::Blink)
    }
}
