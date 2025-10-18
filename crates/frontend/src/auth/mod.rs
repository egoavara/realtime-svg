pub mod storage;
pub mod token;

use crate::auth::storage::{LocalTokenStorage, TokenStorage};
use crate::auth::token::decode_claims;
use yew::prelude::*;

#[derive(Debug, Clone, PartialEq)]
pub enum AuthState {
    Anonymous,
    Authenticated { user_id: String, token: String },
}

pub type AuthContext = UseStateHandle<AuthState>;

#[derive(Properties, PartialEq)]
pub struct AuthProviderProps {
    #[prop_or_default]
    pub children: Children,
}

#[function_component(AuthProvider)]
pub fn auth_provider(props: &AuthProviderProps) -> Html {
    let storage = LocalTokenStorage::new();

    let state = use_state(|| {
        if let Some(token) = storage.get_token() {
            if let Ok(claims) = decode_claims(&token) {
                log::info!("Token loaded from localStorage for user: {}", claims.sub);
                return AuthState::Authenticated {
                    user_id: claims.sub,
                    token,
                };
            }
        }
        AuthState::Anonymous
    });

    html! {
        <ContextProvider<AuthContext> context={state}>
            {props.children.clone()}
        </ContextProvider<AuthContext>>
    }
}
