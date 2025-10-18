use yew::prelude::*;
use web_sys::HtmlInputElement;
use wasm_bindgen_futures::spawn_local;

use crate::api::auth::request_token;
use crate::auth::storage::{TokenStorage, LocalTokenStorage};
use crate::auth::{AuthContext, AuthState};
use crate::auth::token::decode_claims;

#[function_component(LoginForm)]
pub fn login_form() -> Html {
    let user_id_ref = use_node_ref();
    let password_ref = use_node_ref();
    let error_message = use_state(|| None::<String>);
    let loading = use_state(|| false);
    let auth_context = use_context::<AuthContext>().expect("AuthContext must be provided");
    
    let on_submit = {
        let user_id_ref = user_id_ref.clone();
        let password_ref = password_ref.clone();
        let error_message = error_message.clone();
        let loading = loading.clone();
        let auth_context = auth_context.clone();
        
        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            
            let user_id = user_id_ref.cast::<HtmlInputElement>()
                .map(|input| input.value().trim().to_string())
                .unwrap_or_default();
            
            let password = password_ref.cast::<HtmlInputElement>()
                .map(|input| input.value().trim().to_string())
                .unwrap_or_default();
            
            if user_id.is_empty() {
                error_message.set(Some("User ID를 입력하세요".to_string()));
                return;
            }
            
            if password.is_empty() {
                error_message.set(Some("비밀번호를 입력하세요".to_string()));
                return;
            }
            
            let error_message = error_message.clone();
            let loading = loading.clone();
            let auth_context = auth_context.clone();
            
            loading.set(true);
            error_message.set(None);
            
            spawn_local(async move {
                match request_token(user_id.clone(), password, None).await {
                    Ok(token) => {
                        let storage = LocalTokenStorage::new();
                        if let Err(e) = storage.set_token(&token) {
                            error_message.set(Some(format!("토큰 저장 실패: {}", e)));
                            loading.set(false);
                            return;
                        }
                        
                        match decode_claims(&token) {
                            Ok(claims) => {
                                auth_context.set(AuthState::Authenticated { 
                                    user_id: claims.sub,
                                    token 
                                });
                            }
                            Err(e) => {
                                error_message.set(Some(format!("토큰 디코딩 실패: {}", e)));
                            }
                        }
                    }
                    Err(e) => {
                        error_message.set(Some(e));
                    }
                }
                loading.set(false);
            });
        })
    };
    
    html! {
        <div class="login-form">
            <h2>{"로그인"}</h2>
            <form onsubmit={on_submit}>
                <div class="form-group">
                    <label for="user_id">{"User ID:"}</label>
                    <input 
                        type="text" 
                        id="user_id" 
                        ref={user_id_ref}
                        placeholder="사용자 ID를 입력하세요"
                        disabled={*loading}
                    />
                </div>
                
                <div class="form-group">
                    <label for="password">{"비밀번호:"}</label>
                    <input 
                        type="password" 
                        id="password" 
                        ref={password_ref}
                        placeholder="비밀번호를 입력하세요"
                        disabled={*loading}
                    />
                </div>
                
                {if let Some(ref msg) = *error_message {
                    html! { <div class="error">{msg}</div> }
                } else {
                    html! {}
                }}
                
                <button type="submit" disabled={*loading}>
                    {if *loading { "처리 중..." } else { "로그인" }}
                </button>
            </form>
        </div>
    }
}
