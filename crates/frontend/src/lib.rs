use gloo_net::http::{Request, Response};
use js_sys::Date;
use serde_json::{json, Value};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use web_sys::{HtmlInputElement, InputEvent, SubmitEvent};
use yew::classes;
use yew::prelude::*;

const DEFAULT_HEADLINE: &str = "realtime-svg";
const DEFAULT_SUB: &str = "Ready to stream";
const DEFAULT_EXPIRE: &str = "1d";
const DEFAULT_TEMPLATE: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" width="600" height="320" viewBox="0 0 600 320">
  <defs>
    <linearGradient id="bg" x1="0%" y1="0%" x2="100%" y2="100%">
      <stop offset="0%" stop-color="#0f172a"/>
      <stop offset="100%" stop-color="#1e293b"/>
    </linearGradient>
  </defs>
  <rect width="100%" height="100%" fill="url(#bg)" rx="32"/>
  <g font-family="'Segoe UI', system-ui, sans-serif" text-anchor="middle" dominant-baseline="middle">
    <text x="50%" y="45%" font-size="42" fill="#e2e8f0">
      {{ headline | default(value="realtime-svg") }}
    </text>
    <text x="50%" y="65%" font-size="22" fill="#94a3b8" letter-spacing="0.1em">
      {{ sub | default(value="Ready to stream") }}
    </text>
  </g>
</svg>
"##;

#[derive(Clone, PartialEq)]
enum SubmitState {
    Idle,
    Loading,
    Success,
    Error(String),
}

#[function_component(App)]
fn app() -> Html {
    let session_input = use_state(String::default);
    let session_expire = use_state(|| DEFAULT_EXPIRE.to_string());
    let sessions = use_state(Vec::<String>::new);
    let active_session = use_state(|| Option::<String>::None);
    let stream_url = use_state(|| Option::<String>::None);

    let headline = use_state(|| DEFAULT_HEADLINE.to_string());
    let sub = use_state(|| DEFAULT_SUB.to_string());

    let create_status = use_state(|| SubmitState::Idle);
    let update_status = use_state(|| SubmitState::Idle);

    let on_session_input = {
        let session_input = session_input.clone();
        let create_status = create_status.clone();
        Callback::from(move |event: InputEvent| {
            if let Some(input) = event
                .target()
                .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
            {
                session_input.set(input.value());
                create_status.set(SubmitState::Idle);
            }
        })
    };

    let on_headline = {
        let headline = headline.clone();
        Callback::from(move |event: InputEvent| {
            if let Some(input) = event
                .target()
                .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
            {
                headline.set(input.value());
            }
        })
    };

    let on_sub = {
        let sub = sub.clone();
        Callback::from(move |event: InputEvent| {
            if let Some(input) = event
                .target()
                .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
            {
                sub.set(input.value());
            }
        })
    };

    let on_create_session = {
        let session_input = session_input.clone();
        let sessions = sessions.clone();
        let active_session = active_session.clone();
        let stream_url = stream_url.clone();
        let create_status = create_status.clone();
        let update_status = update_status.clone();
        let session_expire = session_expire.clone();
        let headline_state = headline.clone();
        let sub_state = sub.clone();
        Callback::from(move |event: SubmitEvent| {
            event.prevent_default();
            let raw_id = (*session_input).clone();
            let trimmed = raw_id.trim();
            if trimmed.is_empty() {
                create_status.set(SubmitState::Error("세션 ID를 입력하세요".to_string()));
                return;
            }

            create_status.set(SubmitState::Loading);
            let session_input = session_input.clone();
            let session_expire = session_expire.clone();
            let sessions = sessions.clone();
            let active_session = active_session.clone();
            let stream_url = stream_url.clone();
            let create_status = create_status.clone();
            let update_status = update_status.clone();
            let headline_state = headline_state.clone();
            let sub_state = sub_state.clone();
            let session_id = trimmed.to_string();
            let expire = {
                let raw = (*session_expire).clone();
                let trimmed = raw.trim();
                if trimmed.is_empty() {
                    None
                } else {
                    Some(trimmed.to_string())
                }
            };

            let headline_value = (*headline_state).clone();
            let sub_value = (*sub_state).clone();

            spawn_local(async move {
                match create_session(session_id.clone(), expire, headline_value, sub_value).await {
                    Ok(_) => {
                        create_status.set(SubmitState::Success);
                        session_input.set(String::new());
                        session_expire.set(DEFAULT_EXPIRE.to_string());
                        update_status.set(SubmitState::Idle);

                        let mut next = (*sessions).clone();
                        if !next.iter().any(|id| id == &session_id) {
                            next.push(session_id.clone());
                            next.sort();
                        }
                        sessions.set(next);

                        active_session.set(Some(session_id.clone()));
                        stream_url.set(Some(stream_src(&session_id)));
                    }
                    Err(err) => create_status.set(SubmitState::Error(err)),
                }
            });
        })
    };

    let on_select_session = {
        let active_session = active_session.clone();
        let stream_url = stream_url.clone();
        let update_status = update_status.clone();
        Callback::from(move |session_id: String| {
            stream_url.set(Some(stream_src(&session_id)));
            active_session.set(Some(session_id));
            update_status.set(SubmitState::Idle);
        })
    };

    let on_submit = {
        let headline = headline.clone();
        let sub = sub.clone();
        let active_session = active_session.clone();
        let update_status = update_status.clone();
        Callback::from(move |event: SubmitEvent| {
            event.prevent_default();
            let Some(session_id) = (*active_session).clone() else {
                update_status.set(SubmitState::Error("먼저 세션을 선택하세요".to_string()));
                return;
            };

            update_status.set(SubmitState::Loading);
            let headline = (*headline).clone();
            let sub = (*sub).clone();
            let update_status = update_status.clone();

            spawn_local(async move {
                match send_updates(session_id.clone(), headline, sub).await {
                    Ok(_) => update_status.set(SubmitState::Success),
                    Err(err) => update_status.set(SubmitState::Error(err)),
                }
            });
        })
    };

    let create_status_text = match &*create_status {
        SubmitState::Idle => "".to_string(),
        SubmitState::Loading => "세션 생성 중...".to_string(),
        SubmitState::Success => "세션이 생성되었습니다.".to_string(),
        SubmitState::Error(message) => message.clone(),
    };

    let update_status_text = match &*update_status {
        SubmitState::Idle => "".to_string(),
        SubmitState::Loading => "SVG 업데이트 중...".to_string(),
        SubmitState::Success => "업데이트가 완료되었습니다.".to_string(),
        SubmitState::Error(message) => message.clone(),
    };

    let create_status_class = match &*create_status {
        SubmitState::Error(_) => Some("error"),
        SubmitState::Success => Some("success"),
        SubmitState::Loading => Some("loading"),
        SubmitState::Idle => None,
    };

    let update_status_class = match &*update_status {
        SubmitState::Error(_) => Some("error"),
        SubmitState::Success => Some("success"),
        SubmitState::Loading => Some("loading"),
        SubmitState::Idle => None,
    };

    let preview = if let Some(url) = &*stream_url {
        html! {
            <img class="preview" src={url.clone()} alt="실시간 SVG 스트림" />
        }
    } else {
        html! {
            <div class="preview empty">
                {"세션을 생성하거나 선택하면 미리보기가 시작됩니다"}
            </div>
        }
    };

    let update_disabled =
        matches!(*update_status, SubmitState::Loading) || active_session.is_none();

    let on_expire_input = {
        let session_expire = session_expire.clone();
        let create_status = create_status.clone();
        Callback::from(move |event: InputEvent| {
            if let Some(input) = event
                .target()
                .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
            {
                session_expire.set(input.value());
                create_status.set(SubmitState::Idle);
            }
        })
    };
    html! {
        <main class="app">
            <header>
                <h1>{"realtime-svg"}</h1>
                <span>{"multipart/x-mixed-replace"}</span>
            </header>

            <section class="session-manager">
                <form class="session-form" onsubmit={on_create_session}>
                    <input
                        type="text"
                        value={(*session_input).clone()}
                        oninput={on_session_input}
                        placeholder="새 세션 ID"
                    />
                    <input
                        type="text"
                        value={(*session_expire).clone()}
                        oninput={on_expire_input}
                        placeholder="만료 기간 (예: P1D)"
                    />
                    <button type="submit">{"세션 생성"}</button>
                </form>
                <span class={classes!("status-line", create_status_class)}>
                    {create_status_text}
                </span>
                <div class="session-list">
                    {
                        if sessions.is_empty() {
                            html! { <span class="session-empty">{"등록된 세션이 없습니다"}</span> }
                        } else {
                            sessions
                                .iter()
                                .map(|id| {
                                    let id_clone = id.clone();
                                    let on_select_session = on_select_session.clone();
                                    let is_active = active_session
                                        .as_ref()
                                        .map(|current| current == id)
                                        .unwrap_or(false);
                                    html! {
                                        <button
                                            type="button"
                                            class={classes!(
                                                "session-chip",
                                                if is_active { Some("active") } else { None }
                                            )}
                                            onclick={Callback::from(move |_| {
                                                on_select_session.emit(id_clone.clone())
                                            })}
                                        >
                                            {id}
                                        </button>
                                    }
                                })
                                .collect::<Html>()
                        }
                    }
                </div>
            </section>

            <section class="preview-wrapper">
                <div class="preview-header">
                    <span>{"현재 세션"}</span>
                    <strong>
                        {
                            active_session
                                .as_ref()
                                .map(|id| id.as_str())
                                .unwrap_or("선택되지 않음")
                        }
                    </strong>
                </div>
                {preview}
            </section>

            <form class="editor" onsubmit={on_submit}>
                <label>
                    {"헤드라인"}
                    <input
                        type="text"
                        value={(*headline).clone()}
                        oninput={on_headline}
                        placeholder="realtime-svg"
                    />
                </label>
                <label>
                    {"서브 텍스트"}
                    <input
                        type="text"
                        value={(*sub).clone()}
                        oninput={on_sub}
                        placeholder="Ready to stream"
                    />
                </label>
                <button type="submit" disabled={update_disabled}>{"SVG 업데이트"}</button>
            </form>
            <span class={classes!("status-line", update_status_class)}>
                {update_status_text}
            </span>
        </main>
    }
}

async fn create_session(
    session_id: String,
    expire: Option<String>,
    headline: String,
    sub: String,
) -> Result<(), String> {
    let payload = json!({
        "session_id": session_id,
        "template": DEFAULT_TEMPLATE,
        "args": {
            "headline": headline,
            "sub": sub,
        },
        "expire": expire,
    });

    let response = Request::post("/api/session")
        .header("Content-Type", "application/json")
        .json(&payload)
        .map_err(|err| err.to_string())?
        .send()
        .await
        .map_err(|err| err.to_string())?;

    if response.ok() {
        Ok(())
    } else {
        Err(extract_error(response).await)
    }
}

async fn send_updates(session_id: String, headline: String, sub: String) -> Result<(), String> {
    let payload = json!({
        "args": {
            "headline": headline,
            "sub": sub,
        }
    });
    let endpoint = format!("/api/session/{session_id}");

    let response = Request::put(&endpoint)
        .header("Content-Type", "application/json")
        .json(&payload)
        .map_err(|err| err.to_string())?
        .send()
        .await
        .map_err(|err| err.to_string())?;

    if response.ok() {
        Ok(())
    } else {
        Err(extract_error(response).await)
    }
}

async fn extract_error(response: Response) -> String {
    let status = response.status();
    let text = response
        .text()
        .await
        .unwrap_or_else(|_| "요청 실패".to_string());

    if let Ok(json) = serde_json::from_str::<Value>(&text) {
        if let Some(message) = json.get("error").and_then(Value::as_str) {
            return format!("[{status}] {message}");
        }
    }

    format!("[{status}] {text}")
}

fn stream_src(session_id: &str) -> String {
    format!("/stream/{session_id}?rev={}", Date::now())
}

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    wasm_logger::init(wasm_logger::Config::default());
    yew::Renderer::<App>::new().render();
    Ok(())
}
