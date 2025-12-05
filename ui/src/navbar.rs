use dioxus::prelude::*;

#[component]
pub fn Navbar(children: Element) -> Element {
    rsx! {
        nav { class: "fixed top-0 w-full z-50 bg-gray-900/80 backdrop-blur-md border-b border-white/5",
            div { class: "max-w-7xl mx-auto px-4 sm:px-6 lg:px-8",
                div { class: "flex items-center justify-between h-16",
                    div { class: "flex items-center",
                        div { class: "flex-shrink-0",
                            span { class: "text-transparent bg-clip-text bg-gradient-to-r from-teal-400 to-emerald-500 font-bold text-2xl tracking-tight",
                                "SOULBEET"
                            }
                        }
                        div { class: "hidden md:block ml-10",
                            div { class: "flex items-baseline space-x-4", {children.clone()} }
                        }
                    }
                    // Mobile menu placeholder or simplified view for smaller screens if needed
                    div { class: "md:hidden flex items-center space-x-4", {children} }
                }
            }
        }
    }
}
