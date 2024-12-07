use freya::prelude::*;
use lagan::NetworkTablesVersion;

// Catppuccin Machiatto
const THEME: Theme = {
    use Cow::Borrowed;

    Theme {
        name: "Catppuccino Machiatto",
        body: BodyTheme {
            background: Borrowed("#24273a"),
            color: Borrowed("#cad3f5"),
            ..DARK_THEME.body
        },
        menu_container: MenuContainerTheme {
            background: Borrowed("#1e2030"),
            ..DARK_THEME.menu_container
        },
        menu_item: MenuItemTheme {
            font_theme: FontTheme {
                color: Borrowed("#cad3f5"),
            },
            hover_background: Borrowed("#363a4f"),
            ..DARK_THEME.menu_item
        },
        button: ButtonTheme {
            font_theme: FontTheme {
                color: Borrowed("#cad3f5"),
            },
            background: Borrowed("#363a4f"),
            hover_background: Borrowed("#45475a"),
            border_fill: Borrowed("#8087a2"),
            ..DARK_THEME.button
        },
        ..DARK_THEME
    }
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkTablesState {
    None,
    Client(NetworkTablesVersion),
    Server,
}

fn main() {
    launch_with_props(app, "Lagan", (1280.0, 720.0));
}

fn app() -> Element {
    let nt_state = use_signal(|| NetworkTablesState::None);

    rsx! {
        ThemeProvider {
            theme: THEME,
            Body {
                NetworkTablesStateSelector {
                    state: nt_state
                }
            }
        }
    }
}

#[component]
fn DropDownMenu(label: String, children: Element) -> Element {
    let mut show_menu = use_signal(|| false);
    let animations = use_animation(|ctx| {
        ctx.with(
            AnimNum::new(0.0, 180.0)
                .time(600)
                .ease(Ease::Out)
                .function(Function::Circ),
        )
    });
    let arrow_rotation = animations.get();

    let theme = use_get_theme();

    rsx! {
        rect {
            direction: "horizontal",
            cross_align: "center",
            background: "#1e2030",
            corner_radius: "{theme.button.corner_radius}",

            rect {
                padding: "0 10 0 10",
                label {
                    {label}
                }
            }
            Button {
                theme: theme_with!(ButtonTheme {
                    background: "#181926".into(),
                    border_fill: "transparent".into(),
                    margin: "0".into(),
                    height: "40".into(),
                    width: "40".into()
                }),

                onclick: move |_| {
                    show_menu.toggle();
                    if show_menu() {
                        animations.start();
                    } else {
                        animations.reverse();
                    }
                },
                ArrowIcon {
                    rotate: arrow_rotation.read().as_f32().to_string(),
                    fill: "{theme.button.font_theme.color}"
                }
            }
        }
        if show_menu() {
            Menu {
                onclose: move |_| {
                    show_menu.set(false);
                    animations.reverse();
                },
                {children}
            }
        }
    }
}

#[component]
fn NetworkTablesStateSelector(state: Signal<NetworkTablesState, UnsyncStorage>) -> Element {
    let displayed_state = match state() {
        NetworkTablesState::None => "None",
        NetworkTablesState::Client(NetworkTablesVersion::V3) => "Client (V3)",
        NetworkTablesState::Client(NetworkTablesVersion::V4) => "Client (V4)",
        NetworkTablesState::Server => "Server",
    };

    rsx! {
        rect {
            width: "100%",
            height: "100%",
            padding: "10 70 10 70",
            DropDownMenu {
                label: "State: {displayed_state}",
                MenuButton {
                    label {
                        "None"
                    }
                    onclick: move |_| state.set(NetworkTablesState::None)
                }
                MenuButton {
                    label {
                        "Server"
                    }
                    onclick: move |_| state.set(NetworkTablesState::Server)
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
