use dioxus::prelude::*;

#[component]
pub fn Navbar(children: Element) -> Element {
    rsx! {
        nav { class: "bg-gray-900 border-b border-gray-800 sticky top-0 z-50",
            div { class: "container mx-auto px-4",
                div { class: "flex items-center justify-between h-16",
                    div { class: "flex items-center gap-8",
                        div { class: "flex-shrink-0",
                            span { class: "text-teal-400 font-bold text-xl tracking-wider",
                                "SOULFUL"
                            }
                        }
                        div { class: "flex items-baseline space-x-4", {children} }
                    }
                }
            }
        }
    }
}
