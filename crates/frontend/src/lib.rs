mod routes;
mod types;
mod auth;
mod api;
mod components;

use yew::prelude::*;
use yew_router::prelude::*;
use routes::Route;
use auth::{AuthProvider, AuthContext, AuthState};
use components::{LoginForm, Header, SessionForm, SessionMode, SessionDetailPage, SessionListPage};

#[function_component(QuickAccessForm)]
fn quick_access_form() -> Html {
    let navigator = use_navigator().expect("Navigator must be available");
    let session_id_ref = use_node_ref();
    let error_message = use_state(|| None::<String>);
    
    let on_submit = {
        let navigator = navigator.clone();
        let session_id_ref = session_id_ref.clone();
        let error_message = error_message.clone();
        
        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            
            let session_id = session_id_ref.cast::<web_sys::HtmlInputElement>()
                .map(|input| input.value().trim().to_string())
                .unwrap_or_default();
            
            if session_id.is_empty() {
                error_message.set(Some("세션 ID를 입력하세요".to_string()));
                return;
            }
            
            navigator.push(&Route::PublicSession { session_id });
        })
    };
    
    html! {
        <div class="quick-access-form">
            <h2>{"세션 바로가기"}</h2>
            <form onsubmit={on_submit}>
                <div class="form-group">
                    <label for="quick_session_id">{"세션 ID:"}</label>
                    <div class="quick-access-input-group">
                        <input 
                            type="text" 
                            id="quick_session_id" 
                            ref={session_id_ref}
                            placeholder="test.png"
                        />
                        <button type="submit">{"접속"}</button>
                    </div>
                </div>
                
                {if let Some(ref msg) = *error_message {
                    html! { <div class="error">{msg}</div> }
                } else {
                    html! {}
                }}
            </form>
        </div>
    }
}

#[function_component(HomePage)]
fn home_page() -> Html {
    let auth_context = use_context::<AuthContext>().expect("AuthContext must be provided");
    
    html! {
        <div class="home-page">
            {match &*auth_context {
                AuthState::Anonymous => html! {
                    <>
                        <QuickAccessForm />
                        <div style="margin: 2rem 0; text-align: center; color: #94a3b8;">
                            <p>{"또는"}</p>
                        </div>
                        <SessionForm mode={SessionMode::PublicMode} />
                        <div style="margin: 2rem 0; text-align: center; color: #94a3b8;">
                            <p>{"또는"}</p>
                        </div>
                        <LoginForm />
                    </>
                },
                AuthState::Authenticated { user_id, .. } => html! {
                    <div>
                        <h2 style="text-align: center; margin-bottom: 2rem; color: #38bdf8;">
                            {format!("환영합니다, {}님!", user_id)}
                        </h2>
                        <SessionForm mode={SessionMode::UserMode} />
                    </div>
                }
            }}
        </div>
    }
}



#[function_component(NotFoundPage)]
fn not_found_page() -> Html {
    html! {
        <div>
            <h1>{"404 Not Found"}</h1>
            <p>{"페이지를 찾을 수 없습니다"}</p>
            <a href="/">{"홈으로 돌아가기"}</a>
        </div>
    }
}

fn switch(route: Route) -> Html {
    match route {
        Route::Home => html! { <HomePage /> },
        Route::PublicSession { session_id } => {
            html! { 
                <SessionDetailPage 
                    user_id={String::new()} 
                    {session_id} 
                    is_user_session={false} 
                /> 
            }
        }
        Route::UserSession { user_id, session_id } => {
            html! { 
                <SessionDetailPage 
                    {user_id} 
                    {session_id} 
                    is_user_session={true} 
                /> 
            }
        }
        Route::MySessions => html! { <SessionListPage /> },
        Route::NotFound => html! { <NotFoundPage /> },
    }
}

#[function_component(App)]
fn app() -> Html {
    html! {
        <AuthProvider>
            <BrowserRouter>
                <Header />
                <Switch<Route> render={switch} />
            </BrowserRouter>
        </AuthProvider>
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn start() -> Result<(), wasm_bindgen::JsValue> {
    wasm_logger::init(wasm_logger::Config::default());
    yew::Renderer::<App>::new().render();
    Ok(())
}
