use yew::prelude::*;
use yew_router::prelude::*;
use web_sys::HtmlInputElement;
use wasm_bindgen_futures::spawn_local;
use std::collections::HashMap;

use crate::routes::Route;
use crate::auth::{AuthContext, AuthState};
use crate::types::{UserSessionCreateRequest, PublicSessionCreateRequest};
use crate::api::user_session::create_user_session;
use crate::api::public_session::create_public_session;

#[derive(Debug, Clone, PartialEq)]
pub enum SessionMode {
    UserMode,
    PublicMode,
}

#[derive(Properties, PartialEq)]
pub struct SessionFormProps {
    pub mode: SessionMode,
}

#[function_component(SessionForm)]
pub fn session_form(props: &SessionFormProps) -> Html {
    let auth_context = use_context::<AuthContext>().expect("AuthContext must be provided");
    let navigator = use_navigator().expect("Navigator must be available");
    
    let session_id_ref = use_node_ref();
    let template_ref = use_node_ref();
    let args_ref = use_node_ref();
    let expire_ref = use_node_ref();
    
    let error_message = use_state(|| None::<String>);
    let loading = use_state(|| false);
    
    let on_submit = {
        let auth_context = auth_context.clone();
        let navigator = navigator.clone();
        let mode = props.mode.clone();
        let session_id_ref = session_id_ref.clone();
        let template_ref = template_ref.clone();
        let args_ref = args_ref.clone();
        let expire_ref = expire_ref.clone();
        let error_message = error_message.clone();
        let loading = loading.clone();
        
        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            
            let session_id = session_id_ref.cast::<HtmlInputElement>()
                .map(|input| input.value().trim().to_string())
                .unwrap_or_default();
            
            let template = template_ref.cast::<web_sys::HtmlTextAreaElement>()
                .map(|input| input.value().trim().to_string())
                .unwrap_or_default();
            
            let args_str = args_ref.cast::<HtmlInputElement>()
                .map(|input| input.value().trim().to_string())
                .unwrap_or_default();
            
            let expire_str = expire_ref.cast::<HtmlInputElement>()
                .map(|input| input.value().trim().to_string())
                .unwrap_or_default();
            
            if session_id.is_empty() {
                error_message.set(Some("세션 ID를 입력하세요".to_string()));
                return;
            }
            
            if template.is_empty() {
                error_message.set(Some("템플릿을 입력하세요".to_string()));
                return;
            }
            
            let args: Option<HashMap<String, serde_json::Value>> = if args_str.is_empty() {
                None
            } else {
                match serde_json::from_str(&args_str) {
                    Ok(parsed) => Some(parsed),
                    Err(_) => {
                        error_message.set(Some("Args는 유효한 JSON 형식이어야 합니다".to_string()));
                        return;
                    }
                }
            };
            
            let expire_seconds: Option<u64> = if expire_str.is_empty() {
                None
            } else {
                match expire_str.parse() {
                    Ok(val) => Some(val),
                    Err(_) => {
                        error_message.set(Some("TTL은 숫자여야 합니다".to_string()));
                        return;
                    }
                }
            };
            
            let auth_context = auth_context.clone();
            let navigator = navigator.clone();
            let mode = mode.clone();
            let error_message = error_message.clone();
            let loading = loading.clone();
            
            loading.set(true);
            error_message.set(None);
            
            spawn_local(async move {
                match mode {
                    SessionMode::UserMode => {
                        match &*auth_context {
                            AuthState::Anonymous => {
                                error_message.set(Some("로그인이 필요합니다".to_string()));
                                loading.set(false);
                            }
                            AuthState::Authenticated { user_id, .. } => {
                                let request = UserSessionCreateRequest {
                                    session_id: session_id.clone(),
                                    template,
                                    args,
                                    expire_seconds,
                                };
                                
                                match create_user_session(user_id, request).await {
                                    Ok(response) => {
                                        navigator.push(&Route::UserSession {
                                            user_id: response.user_id,
                                            session_id: response.session_id,
                                        });
                                    }
                                    Err(e) => {
                                        error_message.set(Some(e));
                                        loading.set(false);
                                    }
                                }
                            }
                        }
                    }
                    SessionMode::PublicMode => {
                        let request = PublicSessionCreateRequest {
                            session_id: session_id.clone(),
                            template,
                            args,
                            expire: expire_seconds.map(|s| format!("{}s", s)),
                        };
                        
                        spawn_local(async move {
                            match create_public_session(request).await {
                                Ok(session_id) => {
                                    navigator.push(&Route::PublicSession { session_id });
                                }
                                Err(e) => {
                                    error_message.set(Some(e));
                                    loading.set(false);
                                }
                            }
                        });
                    }
                }
            });
        })
    };
    
    let default_template = "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"400\" height=\"200\" viewBox=\"0 0 400 200\">\n  <rect width=\"400\" height=\"200\" fill=\"#0f172a\"/>\n  <text x=\"200\" y=\"100\" text-anchor=\"middle\" font-size=\"24\" fill=\"#38bdf8\">\n    {{text}}\n  </text>\n</svg>";
    
    let default_args = "{\"text\": \"Hello, World!\"}";
    
    html! {
        <div class="session-form">
            <h2>
                {match props.mode {
                    SessionMode::UserMode => "유저 세션 생성",
                    SessionMode::PublicMode => "Public 세션 생성",
                }}
            </h2>
            
            <form onsubmit={on_submit}>
                <div class="form-group">
                    <label for="session_id">{"세션 ID:"}</label>
                    <input 
                        type="text" 
                        id="session_id" 
                        ref={session_id_ref}
                        placeholder="my-session"
                        disabled={*loading}
                    />
                </div>
                
                <div class="form-group">
                    <label for="template">{"템플릿:"}</label>
                    <textarea
                        id="template" 
                        ref={template_ref}
                        value={default_template}
                        disabled={*loading}
                        rows="8"
                    />
                </div>
                
                <div class="form-group">
                    <label for="args">{"Args (JSON):"}</label>
                    <input 
                        type="text" 
                        id="args" 
                        ref={args_ref}
                        value={default_args}
                        disabled={*loading}
                    />
                </div>
                
                <div class="form-group">
                    <label for="expire">{"TTL (초):"}</label>
                    <input 
                        type="number" 
                        id="expire" 
                        ref={expire_ref}
                        value="3600"
                        disabled={*loading}
                    />
                </div>
                
                {if let Some(ref msg) = *error_message {
                    html! { <div class="error">{msg}</div> }
                } else {
                    html! {}
                }}
                
                <button type="submit" disabled={*loading}>
                    {if *loading { "처리 중..." } else { "생성" }}
                </button>
            </form>
        </div>
    }
}
