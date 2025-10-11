use dioxus::prelude::*;

#[derive(Clone, PartialEq, Default)]
pub enum ButtonVariant {
    #[default]
    Primary,
    Secondary,
}

impl ButtonVariant {
    // This function returns the appropriate Tailwind classes for each variant.
    fn get_classes(&self) -> &'static str {
        match self {
            ButtonVariant::Primary => "bg-teal-500 hover:bg-teal-600",
            ButtonVariant::Secondary => "bg-indigo-500 hover:bg-indigo-600",
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct Props {
    children: Element,
    #[props(into)]
    onclick: EventHandler<MouseEvent>,
    #[props(optional, default)]
    variant: ButtonVariant,
    #[props(optional, default)]
    disabled: bool,
}

#[component]
pub fn Button(props: Props) -> Element {
    let common_classes = "text-white font-bold py-2 px-4 rounded-md transition-colors duration-300 disabled:bg-gray-600 disabled:cursor-not-allowed";
    let variant_classes = props.variant.get_classes();

    rsx! {
        button {
            class: "{common_classes} {variant_classes}",
            onclick: move |evt| props.onclick.call(evt),
            disabled: props.disabled,
            {props.children}
        }
    }
}
