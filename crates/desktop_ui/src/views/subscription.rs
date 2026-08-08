use dioxus::prelude::*;

use crate::bridge::AppBridge;

#[component]
pub fn SubscriptionView() -> Element {
    let bridge = use_context::<AppBridge>();
    let settings = bridge.settings.get();

    let privacy_status = if settings.privacy.local_processing_only {
        "Local Only"
    } else if settings.privacy.telemetry_enabled {
        "Standard"
    } else {
        "Privacy-Focused"
    };

    rsx! {
        div {
            div { class: "card",
                div { class: "card-header",
                    span { class: "card-title", "Current Configuration" }
                }
                div { style: "padding: 20px; text-align: center;",
                    div { style: "font-size: 24px; font-weight: 700; color: var(--accent-secondary);",
                        "{settings.app_name}"
                    }
                    div { style: "font-size: 13px; color: var(--text-muted); margin-top: 8px;",
                        "Provider: {settings.models.provider} | Model: {settings.models.model}"
                    }
                }
            }

            div { style: "display: grid; grid-template-columns: repeat(3, 1fr); gap: 16px;",
                div { class: "card",
                    div { style: "text-align: center;",
                        div { style: "font-size: 18px; font-weight: 600;", "Voice" }
                        div { style: "font-size: 14px; color: var(--accent-primary); margin: 12px 0;",
                            if settings.voice.enabled { "Enabled" } else { "Disabled" }
                        }
                        div { style: "font-size: 12px; color: var(--text-secondary); line-height: 1.8;",
                            "Wake: {settings.voice.wake_word}\nLanguage: {settings.voice.language}\nNoise Suppression: {settings.voice.noise_suppression}\nEcho Cancel: {settings.voice.echo_cancellation}"
                        }
                    }
                }

                div { class: "card",
                    div { style: "text-align: center;",
                        div { style: "font-size: 18px; font-weight: 600;", "Memory" }
                        div { style: "font-size: 14px; color: var(--accent-primary); margin: 12px 0;",
                            if settings.memory.enabled { "Enabled" } else { "Disabled" }
                        }
                        div { style: "font-size: 12px; color: var(--text-secondary); line-height: 1.8;",
                            "Max Items: {settings.memory.max_items}\nRetention: {settings.memory.retention_days} days\nAuto Consolidate: {settings.memory.auto_consolidate}\nEmbedding: {settings.memory.embedding_model}"
                        }
                    }
                }

                div { class: "card",
                    div { style: "text-align: center;",
                        div { style: "font-size: 18px; font-weight: 600;", "Automation" }
                        div { style: "font-size: 14px; color: var(--accent-primary); margin: 12px 0;",
                            if settings.automation.enabled { "Enabled" } else { "Disabled" }
                        }
                        div { style: "font-size: 12px; color: var(--text-secondary); line-height: 1.8;",
                            "Consent Required: {settings.automation.require_consent}\nMax Concurrent: {settings.automation.max_concurrent_tasks}\nTimeout: {settings.automation.timeout_seconds}s"
                        }
                    }
                }
            }

            div { class: "card",
                div { class: "card-header",
                    span { class: "card-title", "System Status" }
                }
                div { class: "setting-row",
                    div { class: "setting-label", "GPU Acceleration" }
                    span { class: if settings.performance.gpu_acceleration { "badge badge-success" } else { "badge badge-warning" },
                        if settings.performance.gpu_acceleration { "Enabled" } else { "Disabled" }
                    }
                }
                div { class: "setting-row",
                    div { class: "setting-label", "Privacy Mode" }
                    span { class: "badge badge-info", "{privacy_status}" }
                }
                div { class: "setting-row",
                    div { class: "setting-label", "Max Memory" }
                    span { style: "color: var(--text-secondary);", "{settings.performance.max_memory_mb} MB" }
                }
                div { class: "setting-row",
                    div { class: "setting-label", "Max CPU" }
                    span { style: "color: var(--text-secondary);", "{settings.performance.max_cpu_percent}%" }
                }
                div { class: "setting-row",
                    div { class: "setting-label", "Auto Update" }
                    span { class: if settings.updates.auto_update { "badge badge-success" } else { "badge badge-warning" },
                        if settings.updates.auto_update { "Enabled ({settings.updates.channel})" } else { "Disabled" }
                    }
                }
                div { class: "setting-row",
                    div { class: "setting-label", "Plugins" }
                    span { class: if settings.plugins.enabled { "badge badge-success" } else { "badge badge-warning" },
                        if settings.plugins.enabled { "Enabled (Sandbox: {settings.plugins.sandbox_mode})" } else { "Disabled" }
                    }
                }
            }
        }
    }
}
