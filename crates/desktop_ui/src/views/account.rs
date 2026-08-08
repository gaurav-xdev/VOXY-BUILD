use dioxus::prelude::*;

use crate::bridge::AppBridge;

#[component]
pub fn AccountView() -> Element {
    let bridge = use_context::<AppBridge>();
    let settings = bridge.settings.get();
    let app_name = settings.app_name.clone();

    rsx! {
        div {
            div { class: "card",
                div { class: "card-header",
                    span { class: "card-title", "Profile" }
                }
                div { class: "setting-row",
                    div { class: "setting-label", "Application" }
                    span { style: "color: var(--text-secondary);", "{app_name}" }
                }
                div { class: "setting-row",
                    div { class: "setting-label", "Voice Enabled" }
                    span { style: "color: var(--text-secondary);",
                        if settings.voice.enabled { "Yes" } else { "No" }
                    }
                }
                div { class: "setting-row",
                    div { class: "setting-label", "Memory Enabled" }
                    span { style: "color: var(--text-secondary);",
                        if settings.memory.enabled { "Yes" } else { "No" }
                    }
                }
                div { class: "setting-row",
                    div { class: "setting-label", "Automation Enabled" }
                    span { style: "color: var(--text-secondary);",
                        if settings.automation.enabled { "Yes" } else { "No" }
                    }
                }
                div { class: "setting-row",
                    div { class: "setting-label", "Provider" }
                    span { style: "color: var(--text-secondary);", "{settings.models.provider}" }
                }
                div { class: "setting-row",
                    div { class: "setting-label", "Model" }
                    span { style: "color: var(--text-secondary);", "{settings.models.model}" }
                }
            }

            div { class: "card",
                div { class: "card-header",
                    span { class: "card-title", "Security" }
                }
                div { class: "setting-row",
                    div {
                        div { class: "setting-label", "Guardian Engine" }
                        div { class: "setting-desc", "Security evaluation active" }
                    }
                    span { class: "badge badge-success", "Active" }
                }
                div { class: "setting-row",
                    div {
                        div { class: "setting-label", "Capabilities Registered" }
                        div { class: "setting-desc", "voice:capture, memory:read, memory:write, automation:execute" }
                    }
                    span { class: "badge badge-info", "4" }
                }
            }

            div { class: "card",
                div { class: "card-header",
                    span { class: "card-title", "System" }
                }
                div { class: "setting-row",
                    div { class: "setting-label", "Event Bus" }
                    span { class: "badge badge-success", "Active" }
                }
                div { class: "setting-row",
                    div { class: "setting-label", "Plugin Manager" }
                    span { class: "badge badge-success", "Active" }
                }
                div { class: "setting-row",
                    div { class: "setting-label", "Health Monitor" }
                    span { class: "badge badge-success", "Active" }
                }
                div { class: "setting-row",
                    div { class: "setting-label", "Download Manager" }
                    span { class: "badge badge-success", "Active" }
                }
                div { class: "setting-row",
                    div { class: "setting-label", "Notification Manager" }
                    span { class: "badge badge-success", "Active" }
                }
            }
        }
    }
}
