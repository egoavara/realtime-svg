use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::api::user_session::list_user_sessions;
use crate::auth::{AuthContext, AuthState};
use crate::routes::Route;
use crate::types::SessionListItem;

#[function_component(SessionListPage)]
pub fn session_list_page() -> Html {
    let auth_context = use_context::<AuthContext>().expect("AuthContext must be provided");
    let navigator = use_navigator().expect("Navigator must be available");

    let sessions = use_state(Vec::<SessionListItem>::new);
    let error_message = use_state(|| None::<String>);
    let loading = use_state(|| false);

    {
        let auth_context = auth_context.clone();
        let sessions = sessions.clone();
        let error_message = error_message.clone();
        let loading = loading.clone();

        use_effect_with((), move |_| match &*auth_context {
            AuthState::Authenticated { user_id, .. } => {
                let user_id = user_id.clone();
                loading.set(true);

                spawn_local(async move {
                    match list_user_sessions(&user_id).await {
                        Ok(response) => {
                            sessions.set(response.items);
                        }
                        Err(e) => {
                            error_message.set(Some(e));
                        }
                    }
                    loading.set(false);
                });
            }
            AuthState::Anonymous => {
                error_message.set(Some("로그인이 필요합니다".to_string()));
            }
        });
    }

    let on_session_click = {
        let navigator = navigator.clone();
        let auth_context = auth_context.clone();

        move |session_id: String| {
            let navigator = navigator.clone();
            let auth_context = auth_context.clone();

            Callback::from(move |_: MouseEvent| {
                if let AuthState::Authenticated { user_id, .. } = &*auth_context {
                    navigator.push(&Route::UserSession {
                        user_id: user_id.clone(),
                        session_id: session_id.clone(),
                    });
                }
            })
        }
    };

    let on_new_session = {
        let navigator = navigator.clone();

        Callback::from(move |_: MouseEvent| {
            navigator.push(&Route::Home);
        })
    };

    html! {
        <div class="session-list-page">
            <h2>{"내 세션 목록"}</h2>

            {if *loading {
                html! { <p>{"로딩 중..."}</p> }
            } else if let Some(ref msg) = *error_message {
                html! { <div class="error">{msg}</div> }
            } else if sessions.is_empty() {
                html! {
                    <div class="empty-state">
                        <p>{"생성된 세션이 없습니다"}</p>
                        <button onclick={on_new_session}>{"새 세션 만들기"}</button>
                    </div>
                }
            } else {
                html! {
                    <div class="session-list">
                        {sessions.iter().map(|session| {
                            let session_id = session.session_id.clone();
                            html! {
                                <div
                                    class="session-card"
                                    onclick={on_session_click(session_id.clone())}
                                    key={session_id.clone()}
                                >
                                    <h3>{&session.session_id}</h3>
                                </div>
                            }
                        }).collect::<Html>()}
                    </div>
                }
            }}
        </div>
    }
}
