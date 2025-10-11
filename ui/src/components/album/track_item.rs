use dioxus::prelude::*;
use shared::musicbrainz::Track;

#[derive(Props, Clone, PartialEq)]
pub struct Props {
    track: Track,
    is_selected: bool,
    on_toggle: EventHandler<String>,
}

#[component]
pub fn TrackItem(props: Props) -> Element {
    let track_id = props.track.id.clone();

    rsx! {
      li {
        class: "flex items-center gap-3 p-2 rounded-md cursor-pointer",
        class: if props.is_selected { "bg-teal-800 bg-opacity-50" } else { "hover:bg-gray-700" },
        onclick: move |_| props.on_toggle.call(track_id.clone()),
        div {
          class: "w-5 h-5 border-2 rounded flex items-center justify-center",
          class: if props.is_selected { "border-teal-400 bg-teal-500" } else { "border-gray-500" },
          if props.is_selected {
            "✓"
          }
        }

        span { class: "flex-grow text-gray-300", "{props.track.title}" }
        if let Some(duration) = &props.track.duration {
          span {
            class: "font-mono text-sm",
            class: if props.is_selected { "text-gray-400" } else { "text-gray-500" },
            "{duration}"
          }
        }
      }
    }
}
