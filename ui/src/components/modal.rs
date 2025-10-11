use dioxus::prelude::*;

#[derive(Props, PartialEq, Clone)]
pub struct Props {
    /// A signal to control the visibility of the modal
    pub on_close: EventHandler,
    /// The content to be displayed inside the modal
    pub children: Element,
    /// The header of the modal
    pub header: Element,
}

#[component]
pub fn Modal(props: Props) -> Element {
    rsx! {
      // Backdrop
      div {
        class: "fixed inset-0 bg-black opacity-70 bg-opacity-50 z-40",
        onclick: move |_| props.on_close.call(()),
      }

      // Container
      div {
        class: "fixed inset-0 flex items-center justify-center z-50",
        onclick: move |_| props.on_close.call(()),

        // Content
        div {
          class: "bg-gray-800 max-h-10/12 overflow-auto scrollbar p-4 rounded-lg shadow-xl max-w-lg w-full",
          onclick: move |event| event.stop_propagation(),
          div { class: "flex",
            div { class: "flex-1", {props.header} }
            button {
              class: "text-gray-400 hover:text-white transition-colors",
              onclick: move |_| props.on_close.call(()),
              // Close icon SVG
              svg {
                class: "w-6 h-6",
                fill: "none",
                view_box: "0 0 24 24",
                stroke: "currentColor",
                path {
                  stroke_linecap: "round",
                  stroke_linejoin: "round",
                  stroke_width: "2",
                  d: "M6 18L18 6M6 6l12 12",
                }
              }
            }
          }
          {props.children}
        }
      }
    }
}
