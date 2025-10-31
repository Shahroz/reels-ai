use crate::view::{
    dependency::DependencyList, layout::Wrapper, overview::Overview, stargazer::StargazerList,
};
use dioxus::prelude::*;
use dioxus_router::prelude::*;
// Assuming ChatApp is in the same crate structure relative to this router.
// Adjust path if needed based on actual project structure.
// If `use` is disallowed by strict guidelines, remove this and use FQN below.
// For this plan, assuming `use` is okay for local crate items for brevity in route definition.
use crate::ui::app::ChatApp;


#[derive(Clone, PartialEq, Eq, Routable)]
#[rustfmt::skip]
pub enum Route {
    #[layout(Wrapper)]
        #[route("/")]
        Overview {},
        #[route("/stargazers")]
        StargazerList {},
        #[route("/dependencies")]
        DependencyList {},
            #[route("/chat")]
        ChatApp {},
#[end_layout]
    #[route("/:..segments")]
    PageNotFound { segments: Vec<String> },
}

impl Default for Route {
    fn default() -> Self {
        Self::Overview {}
    }
}

#[component]
fn PageNotFound(segments: Vec<String>) -> Element {
    let path = segments.join("/");
    rsx! {
        div {
            class: "notification is-danger is-light",
            h3 { "Page not found" }
            p { "The page `{path}` you requested doesn't exist." }
        }
    }
}
