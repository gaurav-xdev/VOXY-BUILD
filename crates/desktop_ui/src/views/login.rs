use dioxus::prelude::*;

#[component]
pub fn LoginView() -> Element {
    rsx! {
        div {
            style: "display: flex; align-items: center; justify-content: center; min-height: 100%;",
            div { class: "card", style: "width: 400px; text-align: center;",
                div { style: "font-size: 32px; font-weight: 700; color: var(--accent-secondary); margin-bottom: 8px;", "VOXY" }
                div { style: "font-size: 13px; color: var(--text-muted); margin-bottom: 32px;", "Sign in to your account" }

                div { style: "margin-bottom: 16px;",
                    input { class: "input", placeholder: "Email", r#type: "email" }
                }
                div { style: "margin-bottom: 24px;",
                    input { class: "input", placeholder: "Password", r#type: "password" }
                }
                button { class: "btn btn-primary", style: "width: 100%;", "Sign In" }

                div { style: "margin-top: 16px; font-size: 12px; color: var(--text-muted);",
                    "Don't have an account? "
                    span { style: "color: var(--accent-secondary); cursor: pointer;", "Sign up" }
                }

                div { style: "margin-top: 24px; padding-top: 16px; border-top: 1px solid var(--border);",
                    button { class: "btn btn-secondary", style: "width: 100%;",
                        "Continue as Guest"
                    }
                }
            }
        }
    }
}
