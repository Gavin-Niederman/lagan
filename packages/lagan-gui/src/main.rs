use freya::{hotreload::FreyaCtx, prelude::*};
use lagan::{Client, NetworkTablesVersion};

const THEME: Theme = Theme { ..DARK_THEME };

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkTablesState {
    None,
    Client(NetworkTablesVersion),
    Server(NetworkTablesVersion),
}

fn main() {
    launch_with_props(app, "Lagan", (1280.0, 720.0));
}

fn app() -> Element {
    let nt_state = use_signal(|| NetworkTablesState::None);

    rsx! {
        ThemeProvider {
            theme: THEME,
            NetworkTablesStateSelector {
                state: nt_state
            }
        }
    }
}

#[component]
fn NetworkTablesStateSelector(state: Signal<NetworkTablesState, UnsyncStorage>) -> Element {
    let mut show_menu = use_signal(|| false);

    let displayed_state = match state() {
        NetworkTablesState::None => "None",
        NetworkTablesState::Client(NetworkTablesVersion::V3) => "Client (V3)",
        NetworkTablesState::Client(NetworkTablesVersion::V4) => "Client (V4)",
        NetworkTablesState::Server(NetworkTablesVersion::V3) => "Server (V3)",
        NetworkTablesState::Server(NetworkTablesVersion::V4) => "Server (V4)",
    };

    rsx! {
        rect {
            color: "white",
            background: "rgb(35, 35, 35)",
            height: "100%",
            width: "100%",
            direction: "row",
            label {
                "Current state: {displayed_state}"
            }
            Button {
                onclick: move |_| show_menu.toggle(),
                ArrowIcon {
                    rotate: if show_menu() { "180" } else { "0.0" },
                    fill: "white"
                }
            }

            if show_menu() {
                Menu {
                    onclose: move |_| show_menu.set(false),
                    MenuButton {
                        label {
                            "None"
                        }
                        onclick: move |_| state.set(NetworkTablesState::None)
                    }
                    MenuButton {
                        label {
                            "Server (V3)"
                        }
                        onclick: move |_| state.set(NetworkTablesState::Server(NetworkTablesVersion::V3))
                    }
                    MenuButton {
                        label {
                            "Server (V4)"
                        }
                        onclick: move |_| state.set(NetworkTablesState::Server(NetworkTablesVersion::V4))
                    }
                    MenuButton {
                        label {
                            "Client (V3)"
                        }
                        onclick: move |_| state.set(NetworkTablesState::Client(NetworkTablesVersion::V3))
                    }
                    MenuButton {
                        label {
                            "Client (V4)"
                        }
                        onclick: move |_| state.set(NetworkTablesState::Client(NetworkTablesVersion::V4))
                    }
                }
            }
        }
    }
}
