use crate::ast::{StyleOverrides, Tone, Variant, Value};
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct ResolvedStyles {
    pub bg: String,
    pub text: String,
    pub border: String,
    pub shadow: String,
    pub radius: String,
}

pub fn resolve_design_v1(
    tone: Option<&Tone>,
    variant: Option<&Variant>,
    defaults: (&str, &str, &str, &str, &str), // bg, text, border, shadow, radius
    style: &StyleOverrides,
    props: &HashMap<String, Value>,
) -> ResolvedStyles {
    let mut bg = defaults.0.to_string();
    let mut text = defaults.1.to_string();
    let mut border = defaults.2.to_string();
    let mut shadow = defaults.3.to_string();
    let mut radius = defaults.4.to_string();

    // 1. Apply tone preset
    if let Some(t) = tone {
        match t {
            Tone::Primary => {
                bg = "blue-600".to_string();
                text = "white".to_string();
                border = "blue-700".to_string();
            }
            Tone::Success => {
                bg = "green-50".to_string();
                text = "green-900".to_string();
                border = "green-200".to_string();
            }
            Tone::Warning => {
                bg = "yellow-50".to_string();
                text = "yellow-800".to_string();
                border = "yellow-200".to_string();
            }
            Tone::Danger => {
                bg = "red-50".to_string();
                text = "red-800".to_string();
                border = "red-200".to_string();
            }
            Tone::Info => {
                bg = "blue-50".to_string();
                text = "blue-800".to_string();
                border = "blue-200".to_string();
            }
            Tone::Neutral => {
                bg = "gray-50".to_string();
                text = "gray-800".to_string();
                border = "gray-200".to_string();
            }
        }
    }

    // Apply variant modifiers on top of tone
    if let Some(v) = variant {
        match v {
            Variant::Solid => {
                if let Some(t) = tone {
                    match t {
                        Tone::Primary => { bg = "blue-600".to_string(); text = "white".to_string(); }
                        Tone::Success => { bg = "green-600".to_string(); text = "white".to_string(); }
                        Tone::Warning => { bg = "yellow-600".to_string(); text = "white".to_string(); }
                        Tone::Danger => { bg = "red-600".to_string(); text = "white".to_string(); }
                        Tone::Info => { bg = "blue-600".to_string(); text = "white".to_string(); }
                        Tone::Neutral => { bg = "gray-600".to_string(); text = "white".to_string(); }
                    }
                }
            }
            Variant::Soft => {
                if let Some(t) = tone {
                    match t {
                        Tone::Primary => { bg = "blue-50".to_string(); text = "blue-900".to_string(); }
                        Tone::Success => { bg = "green-50".to_string(); text = "green-900".to_string(); }
                        Tone::Warning => { bg = "yellow-50".to_string(); text = "yellow-900".to_string(); }
                        Tone::Danger => { bg = "red-50".to_string(); text = "red-900".to_string(); }
                        Tone::Info => { bg = "blue-50".to_string(); text = "blue-900".to_string(); }
                        Tone::Neutral => { bg = "gray-50".to_string(); text = "gray-900".to_string(); }
                    }
                }
            }
            Variant::Outline => {
                bg = "transparent".to_string();
                if let Some(t) = tone {
                    match t {
                        Tone::Primary => { text = "blue-600".to_string(); border = "blue-600".to_string(); }
                        Tone::Success => { text = "green-600".to_string(); border = "green-600".to_string(); }
                        Tone::Warning => { text = "yellow-600".to_string(); border = "yellow-600".to_string(); }
                        Tone::Danger => { text = "red-600".to_string(); border = "red-600".to_string(); }
                        Tone::Info => { text = "blue-600".to_string(); border = "blue-600".to_string(); }
                        Tone::Neutral => { text = "gray-600".to_string(); border = "gray-600".to_string(); }
                    }
                }
            }
            Variant::Dark => {
                bg = "gray-950".to_string();
                text = "white".to_string();
                border = "gray-800".to_string();
            }
            Variant::Light => {
                bg = "gray-50".to_string();
                text = "gray-800".to_string();
                border = "gray-200".to_string();
            }
            Variant::Minimal => {
                bg = "transparent".to_string();
                border = "transparent".to_string();
                if let Some(t) = tone {
                    match t {
                        Tone::Primary => { text = "blue-600".to_string(); }
                        Tone::Success => { text = "green-600".to_string(); }
                        Tone::Warning => { text = "yellow-600".to_string(); }
                        Tone::Danger => { text = "red-600".to_string(); }
                        Tone::Info => { text = "blue-600".to_string(); }
                        Tone::Neutral => { text = "gray-600".to_string(); }
                    }
                }
            }
        }
    }

    // 2. Fall back to inline props
    if let Some(val) = props.get("bg").map(|v| v.as_str()) {
        bg = val.to_string();
    }
    if let Some(val) = props.get("text").map(|v| v.as_str()) {
        text = val.to_string();
    }
    if let Some(val) = props.get("border").map(|v| v.as_str()) {
        border = val.to_string();
    }
    if let Some(val) = props.get("shadow").map(|v| v.as_str()) {
        if val.is_empty() || val == "none" {
            shadow = String::new();
        } else if val.starts_with("shadow-") {
            shadow = val.to_string();
        } else {
            shadow = format!("shadow-{}", val);
        }
    }
    if let Some(val) = props.get("radius").map(|v| v.as_str()) {
        radius = val.to_string();
    }

    // 3. Apply style overrides (highest priority)
    if let Some(ref val) = style.bg {
        bg = val.clone();
    }
    if let Some(ref val) = style.text {
        text = val.clone();
    }
    if let Some(ref val) = style.border {
        border = val.clone();
    }
    if let Some(ref val) = style.shadow {
        if val.is_empty() || val == "none" {
            shadow = String::new();
        } else if val.starts_with("shadow-") {
            shadow = val.clone();
        } else {
            shadow = format!("shadow-{}", val);
        }
    }
    if let Some(ref val) = style.radius {
        radius = val.clone();
    }

    ResolvedStyles {
        bg,
        text,
        border,
        shadow,
        radius,
    }
}
