use dioxus::prelude::*;
use std::sync::Arc;
use std::sync::OnceLock;

use tracing_subscriber::EnvFilter;
use voxy_event_bus::EventBus;

struct OverlayState {
    status: parking_lot::RwLock<OverlayStatus>,
    notifications: parking_lot::RwLock<Vec<OverlayNotification>>,
    muted: parking_lot::RwLock<bool>,
}

#[derive(Clone, Debug, PartialEq)]
enum OverlayStatus {
    Idle,
    Listening,
    Speaking,
    Thinking,
}

#[derive(Clone, Debug)]
struct OverlayNotification {
    title: String,
    message: String,
    timestamp: chrono::DateTime<chrono::Utc>,
}

impl OverlayState {
    fn new() -> Self {
        Self {
            status: parking_lot::RwLock::new(OverlayStatus::Idle),
            notifications: parking_lot::RwLock::new(Vec::new()),
            muted: parking_lot::RwLock::new(false),
        }
    }
}

static OVERLAY_STATE: OnceLock<Arc<OverlayState>> = OnceLock::new();

fn get_state() -> Arc<OverlayState> {
    OVERLAY_STATE
        .get_or_init(|| Arc::new(OverlayState::new()))
        .clone()
}

fn status_color(s: &OverlayStatus) -> &'static str {
    match s {
        OverlayStatus::Listening | OverlayStatus::Speaking => "#00d2d3",
        OverlayStatus::Thinking => "#feca57",
        OverlayStatus::Idle => "#576574",
    }
}

fn status_shadow(s: &OverlayStatus) -> &'static str {
    match s {
        OverlayStatus::Listening | OverlayStatus::Speaking => "0 0 12px #00d2d3",
        OverlayStatus::Thinking => "0 0 12px #feca57",
        OverlayStatus::Idle => "none",
    }
}

fn status_label(s: &OverlayStatus) -> &'static str {
    match s {
        OverlayStatus::Listening => "Listening",
        OverlayStatus::Speaking => "Speaking",
        OverlayStatus::Thinking => "Thinking",
        OverlayStatus::Idle => "Idle",
    }
}

fn orb_glow_val(s: &OverlayStatus) -> &'static str {
    match s {
        OverlayStatus::Listening | OverlayStatus::Speaking => "24",
        OverlayStatus::Thinking => "16",
        OverlayStatus::Idle => "8",
    }
}

fn orb_opacity_val(s: &OverlayStatus) -> &'static str {
    match s {
        OverlayStatus::Listening | OverlayStatus::Speaking => "6",
        OverlayStatus::Thinking => "4",
        OverlayStatus::Idle => "2",
    }
}

const ROOT_STYLE: &str = "position: fixed; top: 0; right: 0; width: 320px; height: 100vh; background: rgba(15,15,25,0.92); backdrop-filter: blur(12px); border-left: 1px solid rgba(0,210,211,0.2); display: flex; flex-direction: column; padding: 16px; font-family: Inter, sans-serif; color: #e0e0e0; overflow-y: auto; z-index: 99999;";
const HEADER_ROW: &str = "display: flex; align-items: center; gap: 8px;";
const ORB_CONTAINER: &str = "display: flex; justify-content: center; margin-bottom: 20px;";
const ORB_BASE: &str = "width: 80px; height: 80px; border-radius: 50%; background: radial-gradient(circle at 30% 30%, #00d2d3, #006c75);";
const GRID_STYLE: &str =
    "display: grid; grid-template-columns: 1fr 1fr; gap: 8px; margin-bottom: 16px;";
const STATUS_LABEL_STYLE: &str = "font-size: 12px; font-weight: 600; text-transform: uppercase; letter-spacing: 0.5px; color: #a0a0a0;";
const NOTIF_SECTION: &str = "flex: 1;";
const NOTIF_HEADER: &str = "font-size: 11px; font-weight: 600; text-transform: uppercase; letter-spacing: 0.5px; color: #666; margin-bottom: 8px;";
const NOTIF_EMPTY: &str = "color: #555; font-size: 12px; text-align: center; padding: 20px;";
const NOTIF_CARD: &str =
    "background: rgba(255,255,255,0.05); border-radius: 6px; padding: 8px 10px; font-size: 12px;";
const NOTIF_TITLE: &str = "font-weight: 600; font-size: 11px; color: #00d2d3;";
const NOTIF_MSG: &str = "color: #a0a0a0; margin-top: 2px;";
const BTN_ACTIVE: &str = "background: rgba(0, 210, 211, 0.15); border: 1px solid rgba(0, 210, 211, 0.4); border-radius: 8px; padding: 10px; cursor: pointer; display: flex; flex-direction: column; align-items: center; gap: 4px; font-size: 11px; color: #00d2d3; transition: all 0.2s;";
const BTN_IDLE: &str = "background: rgba(255,255,255,0.03); border: 1px solid rgba(255,255,255,0.08); border-radius: 8px; padding: 10px; cursor: pointer; display: flex; flex-direction: column; align-items: center; gap: 4px; font-size: 11px; color: #a0a0a0; transition: all 0.2s;";
const ICON_STYLE: &str = "font-size: 20px;";
const LABEL_STYLE: &str = "font-weight: 500;";

#[component]
fn App() -> Element {
    let state = get_state();
    let status = use_signal(|| OverlayStatus::Idle);
    let notifications = use_signal(Vec::<OverlayNotification>::new);
    let mut is_muted = use_signal(|| false);

    use_effect(move || {
        let mut status_sig = status.clone();
        let notif_sig = notifications.clone();
        let state_clone = state.clone();
        spawn(async move {
            let bus = Arc::new(EventBus::new(256));

            if let Ok(mut rx) = bus.subscribe("voice.wake").await {
                let s = state_clone.clone();
                let mut ss = status_sig.clone();
                spawn(async move {
                    while let Ok(_event) = rx.recv().await {
                        *s.status.write() = OverlayStatus::Listening;
                        ss.set(OverlayStatus::Listening);
                    }
                });
            }
            if let Ok(mut rx) = bus.subscribe("stt.final").await {
                let s = state_clone.clone();
                let mut ss = status_sig.clone();
                spawn(async move {
                    while let Ok(_event) = rx.recv().await {
                        *s.status.write() = OverlayStatus::Thinking;
                        ss.set(OverlayStatus::Thinking);
                    }
                });
            }
            if let Ok(mut rx) = bus.subscribe("llm.response").await {
                let s = state_clone.clone();
                let mut ss = status_sig.clone();
                spawn(async move {
                    while let Ok(_event) = rx.recv().await {
                        *s.status.write() = OverlayStatus::Speaking;
                        ss.set(OverlayStatus::Speaking);
                    }
                });
            }
            if let Ok(mut rx) = bus.subscribe("tts.audio").await {
                let s = state_clone.clone();
                let mut ss = status_sig.clone();
                spawn(async move {
                    while let Ok(_event) = rx.recv().await {
                        *s.status.write() = OverlayStatus::Speaking;
                        ss.set(OverlayStatus::Speaking);
                    }
                });
            }
            if let Ok(mut rx) = bus.subscribe("desktop.notification.sent").await {
                let s = state_clone.clone();
                let mut ns = notif_sig.clone();
                spawn(async move {
                    while let Ok(event) = rx.recv().await {
                        let notif = OverlayNotification {
                            title: event.source().to_string(),
                            message: String::from_utf8_lossy(event.payload()).to_string(),
                            timestamp: event.timestamp(),
                        };
                        s.notifications.write().push(notif.clone());
                        let mut cur = ns.read().clone();
                        cur.push(notif);
                        ns.set(cur);
                    }
                });
            }

            let s = state_clone;
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                if *s.status.read() != OverlayStatus::Idle {
                    *s.status.write() = OverlayStatus::Idle;
                    status_sig.set(OverlayStatus::Idle);
                }
            }
        });
    });

    let cs = status.read().clone();
    let cm = *is_muted.read();

    let dot_color = status_color(&cs);
    let dot_shadow = status_shadow(&cs);
    let stat_text = status_label(&cs);
    let og = orb_glow_val(&cs);
    let oo = orb_opacity_val(&cs);
    let mute_text = if cm { "Unmute" } else { "Mute" };
    let mute_emoji = if cm { "\u{1f507}" } else { "\u{1f50a}" };
    let mic_active = cs == OverlayStatus::Listening;
    let orb_full = format!(
        "{}; box-shadow: 0 0 {}px rgba(0, 210, 211, 0.{});",
        ORB_BASE, og, oo
    );
    let dot_full = format!(
        "width: 10px; height: 10px; border-radius: 50%; background: {}; box-shadow: 0 0 8px {};",
        dot_color, dot_shadow
    );

    rsx! {
        div { style: ROOT_STYLE,
            div { style: HEADER_ROW,
                div { style: "{dot_full}" }
                span { style: STATUS_LABEL_STYLE,
                    {format_args!("{}", stat_text)}
                }
            }
            div { style: ORB_CONTAINER,
                div { style: "{orb_full}" }
            }
            div { style: GRID_STYLE,
                ActionBtn { label: "Mic", icon: "\u{1f3a4}", active: mic_active, onclick: move |_| { tracing::info!("Mic"); } }
                ActionBtn { label: mute_text, icon: mute_emoji, active: cm, onclick: move |_| { let mut m = is_muted.write(); *m = !*m; } }
                ActionBtn { label: "Settings", icon: "\u{2699}\u{fe0f}", active: false, onclick: move |_| {} }
                ActionBtn { label: "History", icon: "\u{1f4cb}", active: false, onclick: move |_| {} }
            }
            div { style: NOTIF_SECTION,
                div { style: NOTIF_HEADER, "Notifications" }
                for notif in notifications.read().iter() {
                    NotificationCard { title: notif.title.clone(), message: notif.message.clone() }
                }
                if notifications.read().is_empty() {
                    div { style: NOTIF_EMPTY, "No notifications" }
                }
            }
        }
    }
}

#[component]
fn NotificationCard(title: String, message: String) -> Element {
    rsx! {
        div { style: NOTIF_CARD,
            div { style: NOTIF_TITLE, "{title}" }
            div { style: NOTIF_MSG, "{message}" }
        }
    }
}

#[component]
fn ActionBtn(
    label: &'static str,
    icon: &'static str,
    active: bool,
    onclick: EventHandler<MouseEvent>,
) -> Element {
    let btn_style = if active { BTN_ACTIVE } else { BTN_IDLE };
    rsx! {
        button { style: btn_style,
            onclick: onclick,
            span { style: ICON_STYLE, "{icon}" }
            span { style: LABEL_STYLE, "{label}" }
        }
    }
}

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    tracing::info!("Starting VOXY Overlay v{}", env!("CARGO_PKG_VERSION"));

    let _state = OVERLAY_STATE.get_or_init(|| Arc::new(OverlayState::new()));

    dioxus::launch(App);
}
