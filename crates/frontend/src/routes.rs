use yew_router::prelude::*;

#[derive(Debug, Clone, PartialEq, Routable)]
pub enum Route {
    #[at("/")]
    Home,

    #[at("/session/:session_id")]
    PublicSession { session_id: String },

    #[at("/session/:user_id/:session_id")]
    UserSession { user_id: String, session_id: String },

    #[at("/my-sessions")]
    MySessions,

    #[not_found]
    #[at("/404")]
    NotFound,
}
