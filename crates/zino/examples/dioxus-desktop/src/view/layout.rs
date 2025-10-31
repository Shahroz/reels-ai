use crate::router::Route;
use dioxus::prelude::*;
use dioxus_free_icons::icons::{bs_icons::*, fa_brands_icons::FaRust, fa_solid_icons::FaCubes};
use dioxus_router::prelude::*;
use zino_dioxus::prelude::*;

pub fn Wrapper() -> Element {
    rsx! {
        Navbar {
            NavbarStart {
                NavbarLink {
                    to: Route::Overview {},
                    IconText {
                        SvgIcon { shape: BsSpeedometer2 }
                        span { "Overview" }
                    }
                }
                NavbarLink {
                    to: Route::StargazerList {},
                    IconText {
                        SvgIcon { shape: BsStars }
                        span { "Stargazers" }
                    }
                }
                NavbarLink {
                    to: Route::DependencyList {},
                    IconText {
                        SvgIcon { shape: BsBricks }
                        span { "Dependencies" }
                    }
                }
                NavbarLink {
                    to: Route::ChatApp {}, // Assumes Route is imported or FQN used
                    IconText {
                        // Placeholder icon - replace with suitable one if available
                        // e.g., use dioxus_free_icons::icons::bs_icons::BsChatDots
                        span { class: "icon", "💬" } // Simple emoji icon
                        span { "Chat" }
                    }
                }
            }
            NavbarEnd {
                NavbarLink {
                    to: "https://github.com/zino-rs/zino",
                    IconText {
                        SvgIcon { shape: BsGithub }
                        span { "github" }
                    }
                }
                NavbarLink {
                    to: "https://crates.io/crates/zino",
                    IconText {
                        SvgIcon { shape: FaRust }
                        span { "crates.io" }
                    }
                }
                NavbarLink {
                    to: "https://docs.rs/zino",
                    IconText {
                        SvgIcon { shape: FaCubes }
                        span { "docs.rs" }
                    }
                }
            }
        }
        MainContainer {
            Outlet::<Route> {}
        }
    }
}
