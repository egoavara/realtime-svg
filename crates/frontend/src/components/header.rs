use crate::auth::storage::{LocalTokenStorage, TokenStorage};
use crate::auth::{AuthContext, AuthState};
use crate::routes::Route;
use yew::prelude::*;
use yew_router::prelude::*;

#[function_component(Header)]
pub fn header() -> Html {
    let auth_context = use_context::<AuthContext>().expect("AuthContext must be provided");

    let on_logout = {
        let auth_context = auth_context.clone();

        Callback::from(move |_| {
            let storage = LocalTokenStorage::new();
            let _ = storage.remove_token();
            auth_context.set(AuthState::Anonymous);
        })
    };

    html! {
        <header class="app-header">
            <div class="header-content">
                <h1><Link<Route> to={Route::Home}>{"realtime-svg"}</Link<Route>></h1>

                <nav class="header-nav">
                    {match &*auth_context {
                        AuthState::Anonymous => html! {},
                        AuthState::Authenticated { .. } => html! {
                            <Link<Route> to={Route::MySessions} classes="nav-link">
                                {"내 세션 목록"}
                            </Link<Route>>
                        }
                    }}
                </nav>

                <div class="auth-status">
                    {match &*auth_context {
                        AuthState::Anonymous => html! {
                            <span class="status-badge anonymous">{"로그인되지 않음"}</span>
                        },
                        AuthState::Authenticated { user_id, .. } => html! {
                            <>
                                <span class="status-badge authenticated">
                                    {format!("사용자: {}", user_id)}
                                </span>
                                <button class="logout-btn" onclick={on_logout}>
                                    {"로그아웃"}
                                </button>
                            </>
                        }
                    }}
                </div>
            </div>
        </header>
    }
}
