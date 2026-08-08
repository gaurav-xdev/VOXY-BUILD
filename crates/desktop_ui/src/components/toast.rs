use dioxus::prelude::*;

#[derive(Clone, PartialEq)]
pub struct Toast {
    pub id: usize,
    pub kind: ToastKind,
    pub message: String,
}

#[derive(Clone, Copy, PartialEq)]
#[allow(dead_code)]
pub enum ToastKind {
    Info,
    Success,
    Warning,
    Error,
}

#[component]
pub fn ToastContainer(toasts: Vec<Toast>) -> Element {
    rsx! {
        div { class: "toast-container",
            for toast in &toasts {
                div { class: "toast",
                    span {
                        style: "font-size: 14px;",
                        match toast.kind {
                            ToastKind::Info => "\u{2139}",
                            ToastKind::Success => "\u{2705}",
                            ToastKind::Warning => "\u{26A0}",
                            ToastKind::Error => "\u{274C}",
                        }
                    }
                    span { "{toast.message}" }
                }
            }
        }
    }
}
