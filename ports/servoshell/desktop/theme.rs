/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::path::Path;

use egui::{Color32, CornerRadius, Stroke};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum ThemePreset {
    #[default]
    Dark,
    Light,
    HighContrast,
    Nord,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct ExtensionThemeColors {
    pub(crate) frame: Option<Color32>,
    pub(crate) toolbar: Option<Color32>,
    pub(crate) tab_text: Option<Color32>,
    pub(crate) tab_background_text: Option<Color32>,
    pub(crate) omnibox_background: Option<Color32>,
    pub(crate) omnibox_text: Option<Color32>,
    pub(crate) ntp_background: Option<Color32>,
    pub(crate) button_background: Option<Color32>,
}

#[derive(Clone, Debug)]
pub(crate) struct ThemePalette {
    pub(crate) is_dark: bool,
    pub(crate) tab_strip_bg: Color32,
    pub(crate) tab_active_bg: Color32,
    pub(crate) tab_inactive_bg: Color32,
    pub(crate) tab_hover_bg: Color32,
    pub(crate) toolbar_bg: Color32,
    pub(crate) bookmarks_bg: Color32,
    pub(crate) surface_bg: Color32,
    pub(crate) sidebar_bg: Color32,
    pub(crate) card_bg: Color32,
    pub(crate) card_border: Color32,
    pub(crate) input_bg: Color32,
    pub(crate) input_border: Color32,
    pub(crate) accent: Color32,
    pub(crate) accent_soft: Color32,
    pub(crate) text_primary: Color32,
    pub(crate) text_secondary: Color32,
    pub(crate) text_on_accent: Color32,
    pub(crate) success: Color32,
    pub(crate) warning: Color32,
    pub(crate) danger: Color32,
    pub(crate) separator: Color32,
}

impl ThemePalette {
    pub(crate) fn from_preset(preset: &ThemePreset) -> Self {
        match preset {
            ThemePreset::Dark => Self {
                is_dark: true,
                tab_strip_bg: Color32::from_rgb(24, 25, 28),
                tab_active_bg: Color32::from_rgb(44, 46, 51),
                tab_inactive_bg: Color32::from_rgb(31, 33, 37),
                tab_hover_bg: Color32::from_rgb(38, 40, 45),
                toolbar_bg: Color32::from_rgb(44, 46, 51),
                bookmarks_bg: Color32::from_rgb(36, 38, 42),
                surface_bg: Color32::from_rgb(24, 25, 28),
                sidebar_bg: Color32::from_rgb(30, 32, 36),
                card_bg: Color32::from_rgb(36, 38, 44),
                card_border: Color32::from_rgb(56, 60, 68),
                input_bg: Color32::from_rgb(28, 30, 34),
                input_border: Color32::from_rgb(66, 70, 80),
                accent: Color32::from_rgb(16, 185, 129),
                accent_soft: Color32::from_rgba_unmultiplied(16, 185, 129, 38),
                text_primary: Color32::from_rgb(236, 239, 244),
                text_secondary: Color32::from_rgb(156, 163, 175),
                text_on_accent: Color32::WHITE,
                success: Color32::from_rgb(52, 211, 153),
                warning: Color32::from_rgb(251, 191, 36),
                danger: Color32::from_rgb(248, 113, 113),
                separator: Color32::from_rgb(52, 56, 64),
            },
            ThemePreset::Light => Self {
                is_dark: false,
                tab_strip_bg: Color32::from_rgb(228, 231, 236),
                tab_active_bg: Color32::from_rgb(255, 255, 255),
                tab_inactive_bg: Color32::from_rgb(238, 240, 244),
                tab_hover_bg: Color32::from_rgb(245, 247, 250),
                toolbar_bg: Color32::from_rgb(255, 255, 255),
                bookmarks_bg: Color32::from_rgb(246, 247, 249),
                surface_bg: Color32::from_rgb(243, 244, 246),
                sidebar_bg: Color32::from_rgb(236, 238, 242),
                card_bg: Color32::from_rgb(255, 255, 255),
                card_border: Color32::from_rgb(209, 213, 219),
                input_bg: Color32::from_rgb(241, 243, 246),
                input_border: Color32::from_rgb(196, 202, 210),
                accent: Color32::from_rgb(5, 150, 105),
                accent_soft: Color32::from_rgba_unmultiplied(5, 150, 105, 28),
                text_primary: Color32::from_rgb(24, 24, 27),
                text_secondary: Color32::from_rgb(100, 116, 139),
                text_on_accent: Color32::WHITE,
                success: Color32::from_rgb(16, 185, 129),
                warning: Color32::from_rgb(217, 119, 6),
                danger: Color32::from_rgb(220, 38, 38),
                separator: Color32::from_rgb(226, 232, 240),
            },
            ThemePreset::HighContrast => Self {
                is_dark: true,
                tab_strip_bg: Color32::BLACK,
                tab_active_bg: Color32::from_rgb(24, 24, 24),
                tab_inactive_bg: Color32::from_rgb(12, 12, 12),
                tab_hover_bg: Color32::from_rgb(36, 36, 36),
                toolbar_bg: Color32::from_rgb(18, 18, 18),
                bookmarks_bg: Color32::from_rgb(10, 10, 10),
                surface_bg: Color32::BLACK,
                sidebar_bg: Color32::from_rgb(14, 14, 14),
                card_bg: Color32::from_rgb(20, 20, 20),
                card_border: Color32::from_rgb(210, 210, 210),
                input_bg: Color32::BLACK,
                input_border: Color32::WHITE,
                accent: Color32::from_rgb(56, 189, 248),
                accent_soft: Color32::from_rgba_unmultiplied(56, 189, 248, 50),
                text_primary: Color32::WHITE,
                text_secondary: Color32::from_rgb(212, 212, 216),
                text_on_accent: Color32::BLACK,
                success: Color32::from_rgb(74, 222, 128),
                warning: Color32::from_rgb(250, 204, 21),
                danger: Color32::from_rgb(248, 113, 113),
                separator: Color32::from_rgb(120, 120, 120),
            },
            ThemePreset::Nord => Self {
                is_dark: true,
                tab_strip_bg: Color32::from_rgb(46, 52, 64),
                tab_active_bg: Color32::from_rgb(59, 66, 82),
                tab_inactive_bg: Color32::from_rgb(49, 56, 68),
                tab_hover_bg: Color32::from_rgb(67, 76, 94),
                toolbar_bg: Color32::from_rgb(59, 66, 82),
                bookmarks_bg: Color32::from_rgb(52, 59, 73),
                surface_bg: Color32::from_rgb(46, 52, 64),
                sidebar_bg: Color32::from_rgb(51, 57, 70),
                card_bg: Color32::from_rgb(59, 66, 82),
                card_border: Color32::from_rgb(76, 86, 106),
                input_bg: Color32::from_rgb(46, 52, 64),
                input_border: Color32::from_rgb(94, 129, 172),
                accent: Color32::from_rgb(136, 192, 208),
                accent_soft: Color32::from_rgba_unmultiplied(136, 192, 208, 40),
                text_primary: Color32::from_rgb(236, 239, 244),
                text_secondary: Color32::from_rgb(216, 222, 233),
                text_on_accent: Color32::from_rgb(46, 52, 64),
                success: Color32::from_rgb(163, 190, 140),
                warning: Color32::from_rgb(235, 203, 139),
                danger: Color32::from_rgb(191, 97, 106),
                separator: Color32::from_rgb(76, 86, 106),
            },
        }
    }

    pub(crate) fn with_extension_theme(mut self, ext: &ExtensionThemeColors) -> Self {
        if let Some(frame) = ext.frame {
            self.tab_strip_bg = frame;
            self.sidebar_bg = frame;
        }
        if let Some(toolbar) = ext.toolbar {
            self.toolbar_bg = toolbar;
            self.tab_active_bg = toolbar;
            self.bookmarks_bg = blend_color(toolbar, self.tab_strip_bg, 0.35);
            self.card_bg = toolbar;
            self.is_dark = luminance(toolbar) < 0.5;
        }
        if let Some(ntp) = ext.ntp_background {
            self.surface_bg = ntp;
        }
        if let Some(omni_bg) = ext.omnibox_background {
            self.input_bg = omni_bg;
        }
        if let Some(text) = ext.tab_text.or(ext.omnibox_text) {
            self.text_primary = text;
            self.text_secondary = blend_color(text, self.toolbar_bg, 0.35);
        } else if let Some(bg_text) = ext.tab_background_text {
            self.text_primary = bg_text;
        }
        if let Some(btn_bg) = ext.button_background {
            self.accent = btn_bg;
            self.accent_soft = Color32::from_rgba_unmultiplied(btn_bg.r(), btn_bg.g(), btn_bg.b(), 40);
        }
        self.card_border = blend_color(self.text_secondary, self.card_bg, 0.65);
        self.separator = self.card_border;
        self
    }

    pub(crate) fn apply_to_egui(&self, ctx: &egui::Context) {
        let mut visuals = if self.is_dark {
            egui::Visuals::dark()
        } else {
            egui::Visuals::light()
        };

        visuals.panel_fill = self.surface_bg;
        visuals.window_fill = self.card_bg;
        visuals.extreme_bg_color = self.input_bg;
        visuals.faint_bg_color = self.tab_inactive_bg;
        visuals.override_text_color = Some(self.text_primary);
        visuals.selection.bg_fill = self.accent;
        visuals.selection.stroke = Stroke::new(1.0_f32, self.text_on_accent);
        visuals.window_stroke = Stroke::new(1.0_f32, self.card_border);
        visuals.window_corner_radius = CornerRadius::same(10);
        visuals.menu_corner_radius = CornerRadius::same(10);

        visuals.widgets.noninteractive.bg_fill = self.card_bg;
        visuals.widgets.noninteractive.weak_bg_fill = self.card_bg;
        visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, self.separator);
        visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0_f32, self.text_primary);
        visuals.widgets.noninteractive.corner_radius = CornerRadius::same(8);

        visuals.widgets.inactive.bg_fill = self.tab_hover_bg;
        visuals.widgets.inactive.weak_bg_fill = self.tab_hover_bg;
        visuals.widgets.inactive.bg_stroke = Stroke::NONE;
        visuals.widgets.inactive.fg_stroke = Stroke::new(1.0_f32, self.text_primary);
        visuals.widgets.inactive.corner_radius = CornerRadius::same(8);

        visuals.widgets.hovered.bg_fill = blend_color(self.tab_hover_bg, self.text_primary, 0.12);
        visuals.widgets.hovered.weak_bg_fill = blend_color(self.tab_hover_bg, self.text_primary, 0.12);
        visuals.widgets.hovered.bg_stroke = Stroke::new(1.0_f32, self.card_border);
        visuals.widgets.hovered.fg_stroke = Stroke::new(1.0_f32, self.text_primary);
        visuals.widgets.hovered.corner_radius = CornerRadius::same(8);

        visuals.widgets.active.bg_fill = self.accent_soft;
        visuals.widgets.active.weak_bg_fill = self.accent_soft;
        visuals.widgets.active.bg_stroke = Stroke::new(1.0_f32, self.accent);
        visuals.widgets.active.fg_stroke = Stroke::new(1.0_f32, self.text_primary);
        visuals.widgets.active.corner_radius = CornerRadius::same(8);

        ctx.set_visuals(visuals);

        ctx.global_style_mut(|style| {
            style.spacing.item_spacing = egui::vec2(8.0, 8.0);
            style.spacing.button_padding = egui::vec2(14.0, 8.0);
            style.spacing.interact_size = egui::vec2(36.0, 30.0);
        });
    }
}

pub(crate) fn parse_extension_theme(manifest_path: &Path) -> Option<ExtensionThemeColors> {
    let content = std::fs::read_to_string(manifest_path).ok()?;
    let val: serde_json::Value = serde_json::from_str(&content).ok()?;
    let colors = val.get("theme")?.get("colors")?;
    Some(ExtensionThemeColors {
        frame: parse_manifest_color(colors.get("frame")),
        toolbar: parse_manifest_color(colors.get("toolbar")),
        tab_text: parse_manifest_color(colors.get("tab_text")),
        tab_background_text: parse_manifest_color(colors.get("tab_background_text")),
        omnibox_background: parse_manifest_color(
            colors
                .get("omnibox_background")
                .or_else(|| colors.get("toolbar_field")),
        ),
        omnibox_text: parse_manifest_color(
            colors
                .get("omnibox_text")
                .or_else(|| colors.get("toolbar_field_text")),
        ),
        ntp_background: parse_manifest_color(colors.get("ntp_background")),
        button_background: parse_manifest_color(colors.get("button_background")),
    })
}

fn parse_manifest_color(value: Option<&serde_json::Value>) -> Option<Color32> {
    let value = value?;
    if let Some(arr) = value.as_array() {
        let r = arr.first()?.as_u64()? as u8;
        let g = arr.get(1)?.as_u64()? as u8;
        let b = arr.get(2)?.as_u64()? as u8;
        if let Some(a) = arr.get(3).and_then(|v| v.as_f64()) {
            let alpha = if a <= 1.0 {
                (a * 255.0).round() as u8
            } else {
                a as u8
            };
            return Some(Color32::from_rgba_unmultiplied(r, g, b, alpha));
        }
        return Some(Color32::from_rgb(r, g, b));
    }
    if let Some(hex) = value.as_str() {
        let h = hex.trim().trim_start_matches('#');
        if h.len() == 6 {
            let r = u8::from_str_radix(&h[0..2], 16).ok()?;
            let g = u8::from_str_radix(&h[2..4], 16).ok()?;
            let b = u8::from_str_radix(&h[4..6], 16).ok()?;
            return Some(Color32::from_rgb(r, g, b));
        }
    }
    None
}

fn blend_color(a: Color32, b: Color32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    let inv = 1.0 - t;
    Color32::from_rgb(
        (a.r() as f32 * inv + b.r() as f32 * t).round() as u8,
        (a.g() as f32 * inv + b.g() as f32 * t).round() as u8,
        (a.b() as f32 * inv + b.b() as f32 * t).round() as u8,
    )
}

fn luminance(c: Color32) -> f32 {
    (0.2126 * c.r() as f32 + 0.7152 * c.g() as f32 + 0.0722 * c.b() as f32) / 255.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifest_color_rgb_and_hex() {
        let rgb = serde_json::json!([16, 185, 129]);
        assert_eq!(parse_manifest_color(Some(&rgb)), Some(Color32::from_rgb(16, 185, 129)));
        let hex = serde_json::json!("#2e3440");
        assert_eq!(parse_manifest_color(Some(&hex)), Some(Color32::from_rgb(46, 52, 64)));
    }

    #[test]
    fn test_extension_theme_overrides_palette() {
        let base = ThemePalette::from_preset(&ThemePreset::Dark);
        let ext = ExtensionThemeColors {
            frame: Some(Color32::from_rgb(10, 20, 30)),
            toolbar: Some(Color32::from_rgb(240, 240, 240)),
            ..Default::default()
        };
        let themed = base.with_extension_theme(&ext);
        assert_eq!(themed.tab_strip_bg, Color32::from_rgb(10, 20, 30));
        assert_eq!(themed.toolbar_bg, Color32::from_rgb(240, 240, 240));
        assert!(!themed.is_dark);
    }
}
