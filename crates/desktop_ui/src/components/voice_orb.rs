use dioxus::prelude::*;

#[derive(Clone, Copy, PartialEq)]
#[allow(dead_code)]
pub enum OrbState {
    Idle,
    Listening,
    Speaking,
    Thinking,
    Error,
}

#[component]
pub fn VoiceOrb(state: Option<OrbState>, on_click: Option<EventHandler<()>>) -> Element {
    let orb_state = state.unwrap_or(OrbState::Idle);

    let class = match orb_state {
        OrbState::Idle => "orb",
        OrbState::Listening => "orb listening",
        OrbState::Speaking => "orb speaking",
        OrbState::Thinking => "orb thinking",
        OrbState::Error => "orb error",
    };

    let label = match orb_state {
        OrbState::Idle => "Click to start",
        OrbState::Listening => "Listening...",
        OrbState::Speaking => "Speaking...",
        OrbState::Thinking => "Thinking...",
        OrbState::Error => "Error occurred",
    };

    rsx! {
        div { class: "orb-container",
            div {
                class: "{class}",
                onclick: move |_| {
                    if let Some(handler) = &on_click {
                        handler.call(());
                    }
                }
            }
            div {
                style: "margin-top: 24px; font-size: 13px; color: var(--text-muted); text-align: center;",
                "{label}"
            }
        }
    }
}
