use dioxus::prelude::*;
use shared::musicbrainz::Track;

#[derive(Props, PartialEq, Clone)]
pub struct Props {
    pub track: Track,
    pub on_album_click: EventHandler<String>,
}

#[component]
pub fn TrackResult(props: Props) -> Element {
    let track = props.track.clone();

    rsx! {
      div { class: "bg-gray-700 p-4 rounded-lg shadow-md hover:bg-gray-600 transition-colors duration-200",

        div { class: "flex justify-between items-center",

          div {
            h5 { class: "text-lg font-bold text-teal-300", "{track.title}" }
            p { class: "text-md text-gray-300", "{track.artist}" }

            if let (Some(album_title), Some(album_id)) = (&track.album_title, &track.album_id) {
              {
                  let album_id = album_id.clone();
                  rsx! {
                    p {
                      class: "text-sm text-gray-400 italic cursor-pointer hover:text-indigo-300 transition-colors",
                      onclick: move |_| props.on_album_click.call(album_id.clone()),
                      "from \"{album_title}\""
                    }
                  }
              }
            }
          }

          if let Some(duration) = &track.duration {
            p { class: "text-sm font-mono text-gray-400 whitespace-nowrap pl-4",
              "{duration}"
            }
          }
        }
      }
    }
}
