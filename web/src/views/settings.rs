use api::{create_user_folder, get_user_folders, register};
use dioxus::prelude::*;

#[component]
pub fn Settings() -> Element {
    let mut folder_name = use_signal(|| "".to_string());
    let mut folder_path = use_signal(|| "".to_string());
    let mut folders = use_signal(Vec::new);

    let mut new_username = use_signal(|| "".to_string());
    let mut new_password = use_signal(|| "".to_string());

    let mut error = use_signal(|| "".to_string());
    let mut success_msg = use_signal(|| "".to_string());
    let auth = crate::auth::use_auth();

    let fetch_folders = move || async move {
        if let Some(token) = auth.token() {
            match get_user_folders(token).await {
                Ok(fetched_folders) => folders.set(fetched_folders),
                Err(e) => error.set(format!("Failed to fetch folders: {e}")),
            }
        }
    };

    use_future(move || async move {
        fetch_folders().await;
    });

    let handle_add_folder = move |_| async move {
        error.set("".to_string());
        success_msg.set("".to_string());

        if let Some(token) = auth.token() {
            if folder_name().is_empty() || folder_path().is_empty() {
                error.set("Name and Path are required".to_string());
                return;
            }

            match create_user_folder(token, folder_name(), folder_path()).await {
                Ok(_) => {
                    success_msg.set("Folder added successfully".to_string());
                    folder_name.set("".to_string());
                    folder_path.set("".to_string());
                    fetch_folders().await;
                }
                Err(e) => error.set(format!("Failed to add folder: {e}")),
            }
        } else {
            error.set("User not logged in".to_string());
        }
    };

    let handle_create_user = move |_| async move {
        error.set("".to_string());
        success_msg.set("".to_string());

        if new_username().is_empty() || new_password().is_empty() {
            error.set("Username and Password are required".to_string());
            return;
        }

        match register(new_username(), new_password()).await {
            Ok(_) => {
                success_msg.set(format!("User '{}' created successfully", new_username()));
                new_username.set("".to_string());
                new_password.set("".to_string());
            }
            Err(e) => error.set(format!("Failed to create user: {e}")),
        }
    };

    rsx! {
        div { class: "container mx-auto p-4 bg-gray-900 min-h-screen text-white",
            h1 { class: "text-3xl font-bold mb-8 text-teal-400", "Settings" }

            // Folder Management Section
            div { class: "bg-gray-800 p-6 rounded-lg shadow-lg mb-8",
                h2 { class: "text-xl font-semibold mb-4 text-indigo-300", "Manage Music Folders" }

                div { class: "grid grid-cols-1 md:grid-cols-2 gap-4 mb-4",
                    div {
                        label { class: "block text-sm font-medium mb-1",
                            "Folder Name (e.g., 'Music/Common')"
                        }
                        input {
                            class: "w-full p-2 rounded bg-gray-700 border border-gray-600 focus:border-teal-500 focus:outline-none",
                            value: "{folder_name}",
                            oninput: move |e| folder_name.set(e.value()),
                            placeholder: "My Music",
                            "type": "text",
                        }
                    }
                    div {
                        label { class: "block text-sm font-medium mb-1", "Folder Path" }
                        input {
                            class: "w-full p-2 rounded bg-gray-700 border border-gray-600 focus:border-teal-500 focus:outline-none",
                            value: "{folder_path}",
                            oninput: move |e| folder_path.set(e.value()),
                            placeholder: "/home/user/Music",
                            "type": "text",
                        }
                    }
                }

                button {
                    class: "bg-teal-600 hover:bg-teal-700 text-white font-bold py-2 px-4 rounded transition-colors",
                    onclick: handle_add_folder,
                    "Add Folder"
                }
            }
            // Existing Folders List
            div { class: "bg-gray-800 p-6 rounded-lg shadow-lg mb-8",
                h2 { class: "text-xl font-semibold mb-4 text-indigo-300", "Your Folders" }
                if folders.read().is_empty() {
                    p { class: "text-gray-400", "No folders added yet." }
                } else {
                    ul { class: "space-y-2",
                        for folder in folders.read().iter() {
                            li { class: "bg-gray-700 p-3 rounded flex justify-between items-center",
                                span { class: "font-medium text-teal-200", "{folder.name}" }
                                span { class: "text-gray-400 text-sm", "{folder.path}" }
                            }
                        }
                    }
                }
            }

            // User Creation Section
            div { class: "bg-gray-800 p-6 rounded-lg shadow-lg",
                h2 { class: "text-xl font-semibold mb-4 text-indigo-300", "Create New User" }
                div { class: "grid grid-cols-1 md:grid-cols-2 gap-4 mb-4",
                    div {
                        label { class: "block text-sm font-medium mb-1", "New Username" }
                        input {
                            class: "w-full p-2 rounded bg-gray-700 border border-gray-600 focus:border-teal-500 focus:outline-none",
                            value: "{new_username}",
                            oninput: move |e| new_username.set(e.value()),
                            placeholder: "Username",
                            "type": "text",
                        }
                    }
                    div {
                        label { class: "block text-sm font-medium mb-1", "New Password" }
                        input {
                            class: "w-full p-2 rounded bg-gray-700 border border-gray-600 focus:border-teal-500 focus:outline-none",
                            value: "{new_password}",
                            oninput: move |e| new_password.set(e.value()),
                            placeholder: "Password",
                            "type": "password",
                        }
                    }
                }
                button {
                    class: "bg-indigo-600 hover:bg-indigo-700 text-white font-bold py-2 px-4 rounded transition-colors",
                    onclick: handle_create_user,
                    "Create User"
                }
            }
            // Global Messages
            if !error().is_empty() {
                div { class: "mt-4 p-4 bg-red-900/50 border border-red-500 rounded text-red-200",
                    "{error}"
                }
            }
            if !success_msg().is_empty() {
                div { class: "mt-4 p-4 bg-green-900/50 border border-green-500 rounded text-green-200",
                    "{success_msg}"
                }
            }
        }
    }
}
