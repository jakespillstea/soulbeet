use dioxus::prelude::*;

use crate::Button;

#[derive(Props, PartialEq, Clone)]
pub struct Props {
    is_selection_empty: bool,
    on_select: EventHandler,
}

#[component]
pub fn AlbumFooter(props: Props) -> Element {
    rsx! {
      div { class: "p-4 border-t border-gray-700 mt-auto",
        Button {
          disabled: props.is_selection_empty,
          onclick: move |_| props.on_select.call(()),
          "Add Selection to Download Queue"
        }
      }
    }
}
