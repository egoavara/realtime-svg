use std::collections::HashMap;
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlTextAreaElement;
use yew::prelude::*;

use crate::api::public_session::{get_public_session_detail, update_public_session};
use crate::api::user_session::{get_user_session_detail, update_user_session};
use crate::auth::storage::{LocalTokenStorage, TokenStorage};
use crate::auth::{AuthContext, AuthState};
use crate::types::{SessionDetail, SessionUpdateRequest};

#[derive(Properties, PartialEq)]
pub struct SessionDetailPageProps {
    pub user_id: String,
    pub session_id: String,
    pub is_user_session: bool,
}

#[function_component(SessionDetailPage)]
pub fn session_detail_page(props: &SessionDetailPageProps) -> Html {
    let auth_context = use_context::<AuthContext>().expect("AuthContext must be provided");

    let session_detail = use_state(|| None::<SessionDetail>);
    let args_text = use_state(|| String::new());
    let error_message = use_state(|| None::<String>);
    let loading = use_state(|| false);
    let updating = use_state(|| false);
    let toast_message = use_state(|| None::<String>);

    let args_ref = use_node_ref();

    {
        let user_id = props.user_id.clone();
        let session_id = props.session_id.clone();
        let is_user_session = props.is_user_session;
        let session_detail = session_detail.clone();
        let args_text = args_text.clone();
        let error_message = error_message.clone();
        let loading = loading.clone();

        use_effect_with((user_id.clone(), session_id.clone()), move |_| {
            loading.set(true);

            spawn_local(async move {
                let result = if is_user_session {
                    get_user_session_detail(&user_id, &session_id).await
                } else {
                    get_public_session_detail(&session_id).await
                };

                match result {
                    Ok(detail) => {
                        let args_json = serde_json::to_string_pretty(&detail.args)
                            .unwrap_or_else(|_| "{}".to_string());
                        args_text.set(args_json);
                        session_detail.set(Some(detail));
                    }
                    Err(e) => {
                        error_message.set(Some(e));
                    }
                }
                loading.set(false);
            });
        });
    }

    let on_update = {
        let user_id = props.user_id.clone();
        let session_id = props.session_id.clone();
        let is_user_session = props.is_user_session;
        let auth_context = auth_context.clone();
        let args_ref = args_ref.clone();
        let error_message = error_message.clone();
        let updating = updating.clone();
        let session_detail = session_detail.clone();
        let args_text = args_text.clone();

        Callback::from(move |e: MouseEvent| {
            e.prevent_default();

            let args_str = args_ref
                .cast::<HtmlTextAreaElement>()
                .map(|textarea| textarea.value())
                .unwrap_or_default();

            let args: HashMap<String, serde_json::Value> = match serde_json::from_str(&args_str) {
                Ok(parsed) => parsed,
                Err(_) => {
                    error_message.set(Some("Args는 유효한 JSON 형식이어야 합니다".to_string()));
                    return;
                }
            };

            if is_user_session {
                match &*auth_context {
                    AuthState::Anonymous => {
                        error_message.set(Some("로그인이 필요합니다".to_string()));
                        return;
                    }
                    AuthState::Authenticated { .. } => {}
                }
            }

            let user_id = user_id.clone();
            let session_id = session_id.clone();
            let error_message = error_message.clone();
            let updating = updating.clone();
            let auth_context = auth_context.clone();
            let session_detail = session_detail.clone();
            let args_text = args_text.clone();

            updating.set(true);
            error_message.set(None);

            spawn_local(async move {
                let request = SessionUpdateRequest { args: args.clone() };

                let result = if is_user_session {
                    update_user_session(&user_id, &session_id, request).await
                } else {
                    update_public_session(&session_id, request).await
                };

                match result {
                    Ok(_) => {
                        if let Some(mut detail) = (*session_detail).clone() {
                            detail.args = args;
                            let args_json = serde_json::to_string_pretty(&detail.args)
                                .unwrap_or_else(|_| "{}".to_string());
                            args_text.set(args_json);
                            session_detail.set(Some(detail));
                        }
                    }
                    Err(e) => {
                        if is_user_session && e.contains("로그인이 필요합니다") {
                            let storage = LocalTokenStorage::new();
                            let _ = storage.remove_token();
                            auth_context.set(AuthState::Anonymous);
                        }
                        error_message.set(Some(e));
                    }
                }
                updating.set(false);
            });
        })
    };

    let can_edit = if props.is_user_session {
        match &*auth_context {
            AuthState::Authenticated { user_id, .. } => user_id == &props.user_id,
            _ => false,
        }
    } else {
        true
    };

    let stream_url = if props.is_user_session {
        format!("/stream/{}/{}", props.user_id, props.session_id)
    } else {
        format!("/stream/{}", props.session_id)
    };

    html! {
        <div class="session-detail-page">
            <h2>{"세션 상세"}</h2>

            {if let Some(ref msg) = *toast_message {
                html! {
                    <div class="toast-message">
                        {msg}
                    </div>
                }
            } else {
                html! {}
            }}

            {if *loading {
                html! { <p>{"로딩 중..."}</p> }
            } else if let Some(ref detail) = *session_detail {
                html! {
                    <div class="session-content">
                        <div class="stream-preview">
                            <div class="preview-header">
                                <h3>{"실시간 스트림 미리보기"}</h3>
                                <div class="preview-actions">
                                    <button
                                        class="copy-btn"
                                        onclick={{
                                            let stream_url = stream_url.clone();
                                            let toast_message = toast_message.clone();
                                            Callback::from(move |e: MouseEvent| {
                                                e.prevent_default();
                                                if let Some(window) = web_sys::window() {
                                                    let full_url = format!("{}{}",
                                                        window.location().origin().unwrap(),
                                                        stream_url
                                                    );
                                                    let clipboard = window.navigator().clipboard();
                                                    let toast_message = toast_message.clone();
                                                    wasm_bindgen_futures::spawn_local(async move {
                                                        let result = wasm_bindgen_futures::JsFuture::from(
                                                            clipboard.write_text(&full_url)
                                                        ).await;
                                                        if result.is_ok() {
                                                            toast_message.set(Some("링크가 복사되었습니다!".to_string()));
                                                            gloo_timers::callback::Timeout::new(2000, move || {
                                                                toast_message.set(None);
                                                            }).forget();
                                                        }
                                                    });
                                                }
                                            })
                                        }}
                                        title="링크 복사"
                                    >
                                        {"🔗"}
                                    </button>
                                    <button
                                        class="copy-btn"
                                        onclick={{
                                            let stream_url = stream_url.clone();
                                            let toast_message = toast_message.clone();
                                            Callback::from(move |e: MouseEvent| {
                                                e.prevent_default();
                                                if let Some(window) = web_sys::window() {
                                                    let full_url = format!("{}{}",
                                                        window.location().origin().unwrap(),
                                                        stream_url
                                                    );
                                                    let html_code = format!(r#"<img src="{}" alt="realtime-svg" />"#, full_url);
                                                    let clipboard = window.navigator().clipboard();
                                                    let toast_message = toast_message.clone();
                                                    wasm_bindgen_futures::spawn_local(async move {
                                                        let result = wasm_bindgen_futures::JsFuture::from(
                                                            clipboard.write_text(&html_code)
                                                        ).await;
                                                        if result.is_ok() {
                                                            toast_message.set(Some("HTML 코드가 복사되었습니다!".to_string()));
                                                            gloo_timers::callback::Timeout::new(2000, move || {
                                                                toast_message.set(None);
                                                            }).forget();
                                                        }
                                                    });
                                                }
                                            })
                                        }}
                                        title="HTML 코드 복사"
                                    >
                                        {"</>"}
                                    </button>
                                </div>
                            </div>
                            <img src={stream_url.clone()} alt="Session stream" />
                        </div>

                        <div class="args-editor">
                            <h3>{"매개변수 (Args)"}</h3>
                            <textarea
                                ref={args_ref}
                                value={(*args_text).clone()}
                                rows="10"
                                disabled={!can_edit || *updating}
                            />

                            {if can_edit {
                                html! {
                                    <button
                                        onclick={on_update}
                                        disabled={*updating}
                                    >
                                        {if *updating { "업데이트 중..." } else { "업데이트" }}
                                    </button>
                                }
                            } else {
                                html! { <p class="info">{"이 세션을 수정할 권한이 없습니다"}</p> }
                            }}
                        </div>

                        {if let Some(ref msg) = *error_message {
                            html! { <div class="error">{msg}</div> }
                        } else {
                            html! {}
                        }}

                        <div class="template-display">
                            <h3>{"템플릿"}</h3>
                            <pre>{&detail.template}</pre>
                        </div>

                        <div class="session-info">
                            <h3>{"세션 정보"}</h3>
                            {if props.is_user_session && !&props.user_id.is_empty() {
                                html! { <p><strong>{"User ID:"}</strong> {&props.user_id}</p> }
                            } else {
                                html! {}
                            }}
                            <p><strong>{"Session ID:"}</strong> {&props.session_id}</p>
                        </div>
                    </div>
                }
            } else {
                html! {
                    <div>
                        {if let Some(ref msg) = *error_message {
                            html! { <div class="error">{msg}</div> }
                        } else {
                            html! { <p>{"세션을 찾을 수 없습니다"}</p> }
                        }}
                    </div>
                }
            }}
        </div>
    }
}
