use dioxus::prelude::*;
use ui::Search;

#[component]
pub fn Home() -> Element {
    rsx! {
        Search {}
    }
}
