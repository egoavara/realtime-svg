use std::collections::HashMap;

use gloo_net::http::{Request, Response};
use js_sys::{decode_uri_component, encode_uri_component, Date};
use serde::Deserialize;
use serde_json::{json, Map as JsonMap, Value};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use web_sys::{HtmlInputElement, HtmlTextAreaElement, InputEvent, SubmitEvent};
use yew::classes;
use yew::prelude::*;

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
const DETAIL_ARGS_PLACEHOLDER: &str = r#"{
  "headline": "realtime-svg",
  "sub": "Ready to stream"
}"#;

type JsonObject = JsonMap<String, Value>;

#[derive(Clone, PartialEq)]
enum SubmitState {
    Idle,
    Loading,
    Success,
    Error(String),
}

#[derive(Clone, PartialEq)]
enum View {
    Create,
    Detail(String),
}

#[derive(Clone, PartialEq)]
enum LoadState {
    Loading,
    Ready,
    Error(String),
}

#[derive(Debug, Clone, Deserialize)]
struct SessionDetail {
    template: String,
    args: HashMap<String, Value>,
}

#[function_component(App)]
fn app() -> Html {
    let view = use_state(detect_view);

    match &*view {
        View::Create => html! { <CreatePage /> },
        View::Detail(session_id) => html! { <DetailPage session_id={session_id.clone()} /> },
    }
}

#[function_component(CreatePage)]
fn create_page() -> Html {
    let session_id = use_state(String::default);
    let expire = use_state(|| DEFAULT_EXPIRE.to_string());
    let template = use_state(|| DEFAULT_TEMPLATE.to_string());
    let status = use_state(|| SubmitState::Idle);

    let on_session_id = {
        let session_id = session_id.clone();
        let status = status.clone();
        Callback::from(move |event: InputEvent| {
            if let Some(input) = event
                .target()
                .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
            {
                session_id.set(input.value());
                status.set(SubmitState::Idle);
            }
        })
    };

    let on_expire = {
        let expire = expire.clone();
        let status = status.clone();
        Callback::from(move |event: InputEvent| {
            if let Some(input) = event
                .target()
                .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
            {
                expire.set(input.value());
                status.set(SubmitState::Idle);
            }
        })
    };

    let on_template = {
        let template = template.clone();
        let status = status.clone();
        Callback::from(move |event: InputEvent| {
            if let Some(textarea) = event
                .target()
                .and_then(|t| t.dyn_into::<HtmlTextAreaElement>().ok())
            {
                template.set(textarea.value());
                status.set(SubmitState::Idle);
            }
        })
    };

    let on_submit = {
        let session_id = session_id.clone();
        let expire = expire.clone();
        let template = template.clone();
        let status = status.clone();
        Callback::from(move |event: SubmitEvent| {
            event.prevent_default();
            let current_id = (*session_id).clone();
            let trimmed = current_id.trim().to_string();
            if trimmed.is_empty() {
                status.set(SubmitState::Error("세션 ID를 입력하세요".to_string()));
                return;
            }

            let expire_current = (*expire).clone();
            let expire_text = expire_current.trim().to_string();
            let expire_option = if expire_text.is_empty() {
                None
            } else {
                Some(expire_text)
            };
            let template_text = (*template).clone();

            status.set(SubmitState::Loading);
            let status_handle = status.clone();
            spawn_local(async move {
                match create_session(trimmed.clone(), expire_option, template_text).await {
                    Ok(_) => {
                        status_handle.set(SubmitState::Success);
                        if let Err(err) = redirect_to_detail(&trimmed) {
                            status_handle.set(SubmitState::Error(err));
                        }
                    }
                    Err(err) => status_handle.set(SubmitState::Error(err)),
                }
            });
        })
    };

    let status_text = match &*status {
        SubmitState::Idle => "".to_string(),
        SubmitState::Loading => "세션을 생성하는 중...".to_string(),
        SubmitState::Success => "세션이 생성되었습니다. 곧 상세 페이지로 이동합니다.".to_string(),
        SubmitState::Error(err) => err.clone(),
    };

    let status_class = match &*status {
        SubmitState::Error(_) => Some("error"),
        SubmitState::Success => Some("success"),
        SubmitState::Loading => Some("loading"),
        SubmitState::Idle => None,
    };

    html! {
        <main class="app create-view">
            <header>
                <h1>{"realtime-svg"}</h1>
                <span>{"multipart/x-mixed-replace"}</span>
            </header>

            <form class="create-form" onsubmit={on_submit}>
                <div class="inline-inputs">
                    <input
                        type="text"
                        value={(*session_id).clone()}
                        oninput={on_session_id}
                        placeholder="새 세션 ID"
                    />
                    <input
                        type="text"
                        value={(*expire).clone()}
                        oninput={on_expire}
                        placeholder="만료 기간 (예: 1d)"
                    />
                </div>
                <label class="field">
                    {"템플릿 (Tera/HTML)"}
                    <textarea
                        class="template-editor"
                        value={(*template).clone()}
                        oninput={on_template}
                        rows={10}
                        spellcheck="false"
                    />
                </label>
                <button type="submit">{"세션 생성"}</button>
            </form>

            <span class={classes!("status-line", status_class)}>
                {status_text}
            </span>
        </main>
    }
}

#[derive(Properties, PartialEq)]
struct DetailPageProps {
    session_id: String,
}

#[function_component(DetailPage)]
fn detail_page(props: &DetailPageProps) -> Html {
    let template_text = use_state(String::default);
    let args_text = use_state(|| DETAIL_ARGS_PLACEHOLDER.to_string());
    let load_state = use_state(|| LoadState::Loading);
    let update_status = use_state(|| SubmitState::Idle);
    let preview_rev = use_state(new_rev);

    {
        let session_id = props.session_id.clone();
        let template_text = template_text.clone();
        let args_text = args_text.clone();
        let load_state = load_state.clone();
        let update_status = update_status.clone();
        let preview_rev = preview_rev.clone();

        use_effect_with(session_id.clone(), move |session_id| {
            load_state.set(LoadState::Loading);
            update_status.set(SubmitState::Idle);
            preview_rev.set(new_rev());

            let session_id = session_id.clone();
            spawn_local(async move {
                match fetch_session_detail(&session_id).await {
                    Ok(detail) => {
                        template_text.set(detail.template.clone());
                        args_text.set(format_args_pretty(&detail.args));
                        load_state.set(LoadState::Ready);
                    }
                    Err(err) => {
                        load_state.set(LoadState::Error(err));
                    }
                }
            });

            || ()
        });
    }

    let on_args_input = {
        let args_text = args_text.clone();
        let update_status = update_status.clone();
        Callback::from(move |event: InputEvent| {
            if let Some(textarea) = event
                .target()
                .and_then(|t| t.dyn_into::<HtmlTextAreaElement>().ok())
            {
                args_text.set(textarea.value());
                update_status.set(SubmitState::Idle);
            }
        })
    };

    let on_refresh_preview = {
        let preview_rev = preview_rev.clone();
        Callback::from(move |_| preview_rev.set(new_rev()))
    };

    let on_update = {
        let session_id = props.session_id.clone();
        let args_text = args_text.clone();
        let update_status = update_status.clone();
        let preview_rev = preview_rev.clone();
        Callback::from(move |event: SubmitEvent| {
            event.prevent_default();
            let raw = (*args_text).clone();
            let parsed = match parse_args_text(&raw) {
                Ok(map) => map,
                Err(err) => {
                    update_status.set(SubmitState::Error(err));
                    return;
                }
            };

            update_status.set(SubmitState::Loading);
            let args_text = args_text.clone();
            let update_status = update_status.clone();
            let session_id = session_id.clone();
            let preview_rev = preview_rev.clone();
            let payload_args = parsed.clone();
            let pretty_value = Value::Object(parsed);
            let fallback_raw = raw.clone();

            spawn_local(async move {
                match send_updates(session_id, payload_args).await {
                    Ok(_) => {
                        let pretty =
                            serde_json::to_string_pretty(&pretty_value).unwrap_or(fallback_raw);
                        args_text.set(pretty);
                        preview_rev.set(new_rev());
                        update_status.set(SubmitState::Success);
                    }
                    Err(err) => update_status.set(SubmitState::Error(err)),
                }
            });
        })
    };

    let update_status_text = match &*update_status {
        SubmitState::Idle => "".to_string(),
        SubmitState::Loading => "파라미터를 업데이트하는 중...".to_string(),
        SubmitState::Success => "업데이트가 완료되었습니다.".to_string(),
        SubmitState::Error(err) => err.clone(),
    };

    let update_status_class = match &*update_status {
        SubmitState::Error(_) => Some("error"),
        SubmitState::Success => Some("success"),
        SubmitState::Loading => Some("loading"),
        SubmitState::Idle => None,
    };

    let stream_url = stream_src(&props.session_id, preview_rev.as_ref());

    let content = match &*load_state {
        LoadState::Loading => html! {
            <div class="detail-feedback loading">
                {"세션 정보를 불러오는 중입니다..."}
            </div>
        },
        LoadState::Error(message) => {
            let on_go_home = Callback::from(move |_| {
                if let Err(err) = redirect_to_home() {
                    web_sys::console::error_1(&JsValue::from_str(&err));
                }
            });
            html! {
                <div class="detail-feedback error">
                    <p>{format!("세션 정보를 불러오지 못했습니다: {message}")}</p>
                    <button type="button" onclick={on_go_home}>{"새 세션 만들기"}</button>
                </div>
            }
        }
        LoadState::Ready => html! {
            <>
                <section class="detail-grid">
                    <div class="preview-card">
                        <div class="preview-header">
                            <span>{"스트림 미리보기"}</span>
                            <button type="button" onclick={on_refresh_preview.clone()}>
                                {"새로고침"}
                            </button>
                        </div>
                        <img class="preview" src={stream_url.clone()} alt="실시간 SVG 스트림" />
                        <div class="stream-meta">
                            <span>{"스트림 URL"}</span>
                            <code class="stream-url">{stream_path(&props.session_id)}</code>
                        </div>
                    </div>

                    <label class="field template-field">
                        {"현재 템플릿"}
                        <textarea
                            class="template-view"
                            value={(*template_text).clone()}
                            readonly=true
                            rows={14}
                            spellcheck="false"
                        />
                    </label>
                </section>

                <form class="editor" onsubmit={on_update}>
                    <label class="field">
                        {"매개변수 (JSON 오브젝트)"}
                        <textarea
                            class="args-editor"
                            value={(*args_text).clone()}
                            oninput={on_args_input.clone()}
                            rows={12}
                            spellcheck="false"
                        />
                    </label>
                    <button type="submit" disabled={matches!(*update_status, SubmitState::Loading)}>
                        {"세션 파라미터 업데이트"}
                    </button>
                </form>

                <span class={classes!("status-line", update_status_class)}>
                    {update_status_text}
                </span>
            </>
        },
    };

    html! {
        <main class="app detail-view">
            <header>
                <h1>{"realtime-svg"}</h1>
                <span>{"multipart/x-mixed-replace"}</span>
            </header>
            <section class="session-info">
                <div class="info-row">
                    <span>{"현재 세션"}</span>
                    <strong>{props.session_id.as_str()}</strong>
                </div>
            </section>
            {content}
        </main>
    }
}

async fn create_session(
    session_id: String,
    expire: Option<String>,
    template: String,
) -> Result<(), String> {
    let payload = json!({
        "session_id": session_id,
        "template": template,
        "args": Value::Object(JsonObject::new()),
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

async fn send_updates(session_id: String, args: JsonObject) -> Result<(), String> {
    let payload = json!({
        "args": Value::Object(args),
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

async fn fetch_session_detail(session_id: &str) -> Result<SessionDetail, String> {
    let endpoint = format!("/api/session/{session_id}");
    let response = Request::get(&endpoint)
        .send()
        .await
        .map_err(|err| err.to_string())?;

    if response.ok() {
        response
            .json::<SessionDetail>()
            .await
            .map_err(|err| err.to_string())
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

fn detect_view() -> View {
    if let Some(window) = web_sys::window() {
        if let Ok(path) = window.location().pathname() {
            if let Some(session_id) = session_id_from_path(&path) {
                return View::Detail(session_id);
            }
        }
    }
    View::Create
}

fn session_id_from_path(pathname: &str) -> Option<String> {
    let trimmed = pathname.trim_matches('/');
    let mut segments = trimmed.split('/');
    match (segments.next(), segments.next(), segments.next()) {
        (Some("session"), Some(id), None) if !id.is_empty() => {
            decode_uri_component(id).ok().map(|decoded| decoded.into())
        }
        _ => None,
    }
}

fn redirect_to_detail(session_id: &str) -> Result<(), String> {
    let encoded = encode_uri_component(session_id)
        .as_string()
        .unwrap_or_else(|| session_id.to_string());
    let url = format!("/session/{encoded}");
    if let Some(window) = web_sys::window() {
        window
            .location()
            .set_href(&url)
            .map_err(|_| "상세 페이지로 이동하지 못했습니다".to_string())
    } else {
        Err("브라우저 환경을 사용할 수 없습니다".to_string())
    }
}

fn redirect_to_home() -> Result<(), String> {
    if let Some(window) = web_sys::window() {
        window
            .location()
            .set_href("/")
            .map_err(|_| "홈으로 이동하지 못했습니다".to_string())
    } else {
        Err("브라우저 환경을 사용할 수 없습니다".to_string())
    }
}

fn stream_path(session_id: &str) -> String {
    format!("/stream/{session_id}")
}

fn stream_src(session_id: &str, rev: &str) -> String {
    format!("{}?rev={rev}", stream_path(session_id))
}

fn parse_args_text(text: &str) -> Result<JsonObject, String> {
    if text.trim().is_empty() {
        return Ok(JsonObject::new());
    }

    match serde_json::from_str::<Value>(text) {
        Ok(Value::Object(map)) => Ok(map),
        Ok(_) => Err("JSON 객체 형식이어야 합니다.".to_string()),
        Err(err) => Err(format!("JSON 구문 오류: {err}")),
    }
}

fn format_args_pretty(args: &HashMap<String, Value>) -> String {
    if args.is_empty() {
        "{}".to_string()
    } else {
        serde_json::to_string_pretty(args).unwrap_or_else(|_| "{}".to_string())
    }
}

fn new_rev() -> String {
    format!("{:.0}", Date::now())
}

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    wasm_logger::init(wasm_logger::Config::default());
    yew::Renderer::<App>::new().render();
    Ok(())
}
