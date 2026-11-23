/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::HashSet;

use egui::{Color32, CornerRadius, Id, Key, Order, Sense, Stroke, Vec2};
use url::Url;

use crate::desktop::extensions::{
    ExtensionInfo, ExtensionKind, blocked_requests_count, delete_extension,
    install_chrome_extension, install_firefox_extension, install_local_extension,
    list_installed_extensions, set_extension_enabled, sync_adblock_settings,
};
use crate::desktop::settings::{
    BrowserSettings, BrowserSurface, GuiText, LIBRENAV_BOOKMARKS_FILE, LIBRENAV_HISTORY_FILE,
    LIBRENAV_SETTINGS_FILE, extensions_directory_path, save_bookmarks, save_json_profile, t,
};
use crate::desktop::theme::{ThemePalette, ThemePreset};
use crate::running_app_state::UserInterfaceCommand;
use crate::window::ServoShellWindow;

#[allow(clippy::too_many_arguments)]
pub(crate) fn show_browser_surface(
    ctx: &egui::Context,
    palette: &ThemePalette,
    surface: &mut BrowserSurface,
    settings: &mut BrowserSettings,
    history: &mut Vec<String>,
    bookmarks: &mut HashSet<String>,
    ext_install_input: &mut String,
    ext_install_status: &mut Option<(bool, String)>,
    window: &ServoShellWindow,
) -> bool {
    if *surface == BrowserSurface::None {
        return false;
    }

    let active_surface = *surface;
    let mut requested_surface = active_surface;
    let mut close = false;
    let mut settings_changed = false;
    let mut history_changed = false;
    let mut bookmarks_changed = false;
    let content_rect = ctx.content_rect();
    let lang_str = settings.language.clone();
    let lang = lang_str.as_str();

    if ctx.input(|i| i.key_pressed(Key::Escape)) {
        close = true;
    }

    egui::Area::new(Id::new("librenav-browser-surface"))
        .order(Order::Foreground)
        .fixed_pos(content_rect.min)
        .show(ctx, |ui| {
            egui::Frame::NONE
                .fill(palette.surface_bg)
                .inner_margin(egui::Margin::same(20))
                .show(ui, |ui| {
                    ui.set_min_size(content_rect.size() - Vec2::splat(40.0));
                    render_header(ui, palette, active_surface, lang, &mut close);
                    ui.add_space(14.0);
                    ui.separator();
                    ui.add_space(14.0);

                    ui.horizontal_top(|ui| {
                        render_sidebar(
                            ui,
                            palette,
                            active_surface,
                            lang,
                            &mut requested_surface,
                        );
                        ui.add_space(16.0);

                        egui::ScrollArea::vertical()
                            .auto_shrink([false, false])
                            .show(ui, |ui| {
                                let target_width = ui.available_width().min(900.0);
                                ui.set_width(target_width);
                                match active_surface {
                                    BrowserSurface::Settings => {
                                        render_settings(
                                            ui,
                                            palette,
                                            settings,
                                            lang,
                                            window,
                                            &mut settings_changed,
                                        );
                                    },
                                    BrowserSurface::Extensions => {
                                        render_extensions(
                                            ui,
                                            palette,
                                            settings,
                                            lang,
                                            ext_install_input,
                                            ext_install_status,
                                            window,
                                            &mut close,
                                            &mut settings_changed,
                                        );
                                    },
                                    BrowserSurface::History => {
                                        render_history(
                                            ui,
                                            palette,
                                            history,
                                            lang,
                                            window,
                                            &mut close,
                                            &mut history_changed,
                                        );
                                    },
                                    BrowserSurface::Bookmarks => {
                                        render_bookmarks(
                                            ui,
                                            palette,
                                            bookmarks,
                                            lang,
                                            window,
                                            &mut close,
                                            &mut bookmarks_changed,
                                        );
                                    },
                                    BrowserSurface::None => {},
                                }
                            });
                    });
                });
        });

    if settings_changed {
        sync_adblock_settings(settings);
        save_json_profile(LIBRENAV_SETTINGS_FILE, settings);
    }
    if history_changed {
        save_json_profile(LIBRENAV_HISTORY_FILE, history);
    }
    if bookmarks_changed {
        save_bookmarks(bookmarks);
        let _ = LIBRENAV_BOOKMARKS_FILE;
    }

    if close {
        *surface = BrowserSurface::None;
    } else if requested_surface != active_surface {
        *surface = requested_surface;
    }

    settings_changed
}

fn render_header(
    ui: &mut egui::Ui,
    palette: &ThemePalette,
    active_surface: BrowserSurface,
    lang: &str,
    close: &mut bool,
) {
    ui.horizontal(|ui| {
        let back_btn = ui
            .add(
                egui::Button::new(
                    egui::RichText::new("←")
                        .size(16.0)
                        .color(palette.accent)
                        .strong(),
                )
                .fill(palette.card_bg)
                .stroke(Stroke::new(1.0_f32, palette.card_border))
                .min_size(Vec2::new(36.0, 32.0))
                .corner_radius(CornerRadius::same(8)),
            )
            .on_hover_text(t(GuiText::BackToBrowser, lang));
        if back_btn.clicked() {
            *close = true;
        }
        ui.add_space(10.0);
        let surface_title = match active_surface {
            BrowserSurface::Settings => t(GuiText::Settings, lang),
            BrowserSurface::History => t(GuiText::History, lang),
            BrowserSurface::Bookmarks => t(GuiText::Bookmarks, lang),
            BrowserSurface::Extensions => t(GuiText::Extensions, lang),
            BrowserSurface::None => "",
        };
        ui.label(
            egui::RichText::new(format!("LibreNav  ›  {surface_title}"))
                .size(18.0)
                .color(palette.text_primary)
                .strong(),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui
                .add(
                    egui::Button::new(
                        egui::RichText::new("✕")
                            .size(15.0)
                            .color(palette.text_secondary),
                    )
                    .fill(palette.card_bg)
                    .stroke(Stroke::new(1.0_f32, palette.card_border))
                    .min_size(Vec2::new(32.0, 32.0))
                    .corner_radius(CornerRadius::same(8)),
                )
                .on_hover_text(t(GuiText::Close, lang))
                .clicked()
            {
                *close = true;
            }
        });
    });
}

fn render_sidebar(
    ui: &mut egui::Ui,
    palette: &ThemePalette,
    active_surface: BrowserSurface,
    lang: &str,
    requested_surface: &mut BrowserSurface,
) {
    egui::Frame::NONE
        .fill(palette.sidebar_bg)
        .stroke(Stroke::new(1.0_f32, palette.card_border))
        .corner_radius(CornerRadius::same(10))
        .inner_margin(egui::Margin::same(10))
        .show(ui, |ui| {
            ui.set_width(220.0);
            let items = [
                (BrowserSurface::Settings, t(GuiText::Settings, lang)),
                (BrowserSurface::Extensions, t(GuiText::Extensions, lang)),
                (BrowserSurface::History, t(GuiText::History, lang)),
                (BrowserSurface::Bookmarks, t(GuiText::Bookmarks, lang)),
            ];
            for (surf, label) in items {
                let is_active = active_surface == surf;
                let fill = if is_active {
                    palette.accent_soft
                } else {
                    Color32::TRANSPARENT
                };
                let stroke = if is_active {
                    Stroke::new(1.0_f32, palette.accent)
                } else {
                    Stroke::NONE
                };
                let text_color = if is_active {
                    palette.accent
                } else {
                    palette.text_primary
                };
                let btn = egui::Button::new(
                    egui::RichText::new(format!("  {label}"))
                        .size(14.0)
                        .color(text_color)
                        .strong(),
                )
                .fill(fill)
                .stroke(stroke)
                .corner_radius(CornerRadius::same(8));
                if ui.add_sized([216.0, 36.0], btn).clicked() {
                    *requested_surface = surf;
                }
                ui.add_space(4.0);
            }
            ui.add_space(16.0);
            ui.separator();
            ui.add_space(8.0);
            ui.label(
                egui::RichText::new("LibreNav 0.7  •  Servo")
                    .size(12.0)
                    .color(palette.text_secondary),
            );
        });
}

fn render_settings(
    ui: &mut egui::Ui,
    palette: &ThemePalette,
    settings: &mut BrowserSettings,
    lang: &str,
    window: &ServoShellWindow,
    settings_changed: &mut bool,
) {
    ui.label(
        egui::RichText::new(t(GuiText::Settings, lang))
            .size(22.0)
            .color(palette.text_primary)
            .strong(),
    );
    ui.label(
        egui::RichText::new(t(GuiText::PrivacyDesc, lang))
            .size(13.0)
            .color(palette.text_secondary),
    );
    ui.add_space(14.0);

    // Language Card
    ui_card(ui, palette, |ui| {
        ui_section_title(ui, palette, t(GuiText::Language, lang));
        ui.add_space(8.0);
        ui.horizontal_wrapped(|ui| {
            for (code, label) in [
                ("fr", "Français"),
                ("en", "English"),
                ("eo", "Esperanto"),
                ("es", "Español"),
            ] {
                let is_active = settings.language == code;
                let btn = egui::Button::new(
                    egui::RichText::new(label)
                        .size(13.5)
                        .color(if is_active {
                            palette.text_on_accent
                        } else {
                            palette.text_primary
                        })
                        .strong(),
                )
                .fill(if is_active {
                    palette.accent
                } else {
                    palette.input_bg
                })
                .stroke(Stroke::new(
                    1.0_f32,
                    if is_active {
                        palette.accent
                    } else {
                        palette.card_border
                    },
                ))
                .corner_radius(CornerRadius::same(8));
                if ui.add(btn).clicked() {
                    settings.language = code.into();
                    *settings_changed = true;
                    if let Some(wv) = window.active_webview()
                        && let Some(u) = wv.url()
                        && u.scheme() == "servo"
                        && u.path() == "newtab"
                        && let Ok(newtab_url) = Url::parse(&format!("servo:newtab?lang={code}"))
                    {
                        wv.load(newtab_url);
                    }
                }
            }
        });
    });

    ui.add_space(12.0);

    // Appearance & Theme Card
    ui_card(ui, palette, |ui| {
        ui_section_title(ui, palette, t(GuiText::Appearance, lang));
        ui.add_space(6.0);
        ui.label(
            egui::RichText::new(t(GuiText::ThemeLabel, lang))
                .size(13.0)
                .color(palette.text_secondary),
        );
        ui.add_space(8.0);

        ui.horizontal_wrapped(|ui| {
            let presets = [
                (ThemePreset::Dark, t(GuiText::ThemeDark, lang)),
                (ThemePreset::Light, t(GuiText::ThemeLight, lang)),
                (
                    ThemePreset::HighContrast,
                    t(GuiText::ThemeHighContrast, lang),
                ),
                (ThemePreset::Nord, t(GuiText::ThemeNord, lang)),
            ];
            for (preset, label) in presets {
                let preview = ThemePalette::from_preset(&preset);
                let is_active = settings.active_theme_extension.is_none()
                    && settings.theme_preset == preset;
                let frame = egui::Frame::NONE
                    .fill(preview.card_bg)
                    .stroke(Stroke::new(
                        if is_active { 2.0_f32 } else { 1.0_f32 },
                        if is_active {
                            palette.accent
                        } else {
                            palette.card_border
                        },
                    ))
                    .corner_radius(CornerRadius::same(8))
                    .inner_margin(egui::Margin::symmetric(12, 8));
                let resp = frame
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            for color in [preview.tab_strip_bg, preview.toolbar_bg, preview.accent] {
                                let (rect, _) = ui.allocate_exact_size(
                                    Vec2::splat(12.0),
                                    Sense::hover(),
                                );
                                ui.painter().circle_filled(rect.center(), 5.5, color);
                            }
                            ui.add_space(4.0);
                            ui.label(
                                egui::RichText::new(label)
                                    .size(13.0)
                                    .color(preview.text_primary)
                                    .strong(),
                            );
                        });
                    })
                    .response
                    .interact(Sense::click());
                if resp.clicked() {
                    settings.theme_preset = preset;
                    settings.active_theme_extension = None;
                    *settings_changed = true;
                }
            }
        });

        let installed = list_installed_extensions(lang);
        let theme_exts: Vec<&ExtensionInfo> = installed
            .iter()
            .filter(|e| e.kind == ExtensionKind::Theme && e.enabled)
            .collect();
        if !theme_exts.is_empty() {
            ui.add_space(10.0);
            ui.separator();
            ui.add_space(8.0);
            for ext in theme_exts {
                let is_ext_active =
                    settings.active_theme_extension.as_deref() == Some(ext.id.as_str());
                ui.horizontal(|ui| {
                    ui_badge(ui, t(GuiText::ThemeBadge, lang), palette.accent_soft, palette.accent);
                    ui.label(
                        egui::RichText::new(&ext.name)
                            .color(palette.text_primary)
                            .strong(),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if is_ext_active {
                            if ui.button(t(GuiText::ResetTheme, lang)).clicked() {
                                settings.active_theme_extension = None;
                                *settings_changed = true;
                            }
                        } else if ui.button(t(GuiText::ApplyTheme, lang)).clicked() {
                            settings.active_theme_extension = Some(ext.id.clone());
                            *settings_changed = true;
                        }
                    });
                });
            }
        }

        ui.add_space(10.0);
        ui.separator();
        ui.add_space(8.0);

        *settings_changed |= ui_toggle(
            ui,
            palette,
            &mut settings.show_home_button,
            t(GuiText::ShowHomeButton, lang),
        );
        ui.add_space(6.0);
        *settings_changed |= ui_toggle(
            ui,
            palette,
            &mut settings.show_bookmarks_bar,
            t(GuiText::ShowBookmarksBar, lang),
        );
        ui.add_space(6.0);
        *settings_changed |= ui_toggle(
            ui,
            palette,
            &mut settings.content_scripts_enabled,
            t(GuiText::ContentScriptsLabel, lang),
        );
        ui.label(
            egui::RichText::new(t(GuiText::ContentScriptsDesc, lang))
                .size(12.0)
                .color(palette.text_secondary),
        );
    });

    ui.add_space(12.0);

    // Homepage & Search Engine Card
    ui_card(ui, palette, |ui| {
        ui_section_title(ui, palette, t(GuiText::Homepage, lang));
        ui.label(
            egui::RichText::new(t(GuiText::HomepageDesc, lang))
                .size(12.5)
                .color(palette.text_secondary),
        );
        ui.add_space(6.0);
        *settings_changed |= ui
            .add(
                egui::TextEdit::singleline(&mut settings.homepage)
                    .desired_width(f32::INFINITY),
            )
            .changed();
        if !settings.homepage_is_valid() {
            ui.colored_label(palette.danger, t(GuiText::InvalidHomepage, lang));
        }

        ui.add_space(12.0);
        ui_section_title(ui, palette, t(GuiText::SearchEngine, lang));
        ui.label(
            egui::RichText::new(t(GuiText::SearchEngineDesc, lang))
                .size(12.5)
                .color(palette.text_secondary),
        );
        ui.add_space(6.0);
        *settings_changed |= ui
            .add(
                egui::TextEdit::singleline(&mut settings.searchpage)
                    .desired_width(f32::INFINITY),
            )
            .changed();
        if !settings.searchpage_is_valid() {
            ui.colored_label(palette.danger, t(GuiText::InvalidSearchpage, lang));
        }
    });
}

#[allow(clippy::too_many_arguments)]
fn render_extensions(
    ui: &mut egui::Ui,
    palette: &ThemePalette,
    settings: &mut BrowserSettings,
    lang: &str,
    ext_install_input: &mut String,
    ext_install_status: &mut Option<(bool, String)>,
    window: &ServoShellWindow,
    close: &mut bool,
    settings_changed: &mut bool,
) {
    ui.label(
        egui::RichText::new(t(GuiText::ExtensionsTitle, lang))
            .size(22.0)
            .color(palette.text_primary)
            .strong(),
    );
    ui.label(
        egui::RichText::new(t(GuiText::ExtensionsSubtitle, lang))
            .size(13.0)
            .color(palette.text_secondary),
    );
    ui.add_space(14.0);

    // Built-in uBlock Origin card
    ui_card(ui, palette, |ui| {
        ui.horizontal(|ui| {
            ui_badge(ui, "uBlock Origin", palette.accent_soft, palette.accent);
            ui.label(
                egui::RichText::new("v1.75.0")
                    .size(13.0)
                    .color(palette.text_secondary),
            );
            let blocked = blocked_requests_count();
            ui_badge(
                ui,
                &format!("{}: {blocked}", t(GuiText::BlockedRequestsLabel, lang)),
                palette.input_bg,
                palette.text_primary,
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Dashboard").clicked() {
                    window.queue_user_interface_command(UserInterfaceCommand::Go(
                        "servo:ublock-dashboard".into(),
                    ));
                    *close = true;
                }
                if ui.button(t(GuiText::UblockOpenPopup, lang)).clicked() {
                    window.queue_user_interface_command(UserInterfaceCommand::Go(
                        "servo:ublock".into(),
                    ));
                    *close = true;
                }
            });
        });
        ui.add_space(8.0);
        *settings_changed |= ui_toggle(
            ui,
            palette,
            &mut settings.ublock_enabled,
            t(GuiText::UblockPowerOn, lang),
        );
    });

    ui.add_space(12.0);

    // Install card
    ui_card(ui, palette, |ui| {
        ui_section_title(ui, palette, t(GuiText::InstallExtension, lang));
        ui.add_space(6.0);
        ui.add(
            egui::TextEdit::singleline(ext_install_input)
                .desired_width(f32::INFINITY)
                .hint_text(t(GuiText::ExtensionInputPlaceholder, lang)),
        );
        ui.add_space(8.0);
        ui.horizontal_wrapped(|ui| {
            if ui.button(t(GuiText::InstallFromChromeStore, lang)).clicked() {
                let input = ext_install_input.trim().to_string();
                if !input.is_empty() {
                    match install_chrome_extension(&input, lang) {
                        Ok(msg) => {
                            *ext_install_status = Some((true, msg));
                            ext_install_input.clear();
                        },
                        Err(e) => *ext_install_status = Some((false, e)),
                    }
                }
            }
            if ui
                .button(t(GuiText::InstallFromFirefoxAddons, lang))
                .clicked()
            {
                let input = ext_install_input.trim().to_string();
                if !input.is_empty() {
                    match install_firefox_extension(&input, lang) {
                        Ok(msg) => {
                            *ext_install_status = Some((true, msg));
                            ext_install_input.clear();
                        },
                        Err(e) => *ext_install_status = Some((false, e)),
                    }
                }
            }
            if ui.button(t(GuiText::InstallLocalFile, lang)).clicked() {
                let input = ext_install_input.trim().to_string();
                if !input.is_empty() {
                    match install_local_extension(&input) {
                        Ok(msg) => {
                            *ext_install_status = Some((true, msg));
                            ext_install_input.clear();
                        },
                        Err(e) => *ext_install_status = Some((false, e)),
                    }
                }
            }
        });

        if let Some((success, msg)) = ext_install_status.as_ref() {
            ui.add_space(6.0);
            let color = if *success {
                palette.success
            } else {
                palette.danger
            };
            let prefix = if *success { "✓" } else { "✕" };
            ui.colored_label(color, format!("{prefix} {msg}"));
        }
    });

    ui.add_space(12.0);

    // Popular 1-click extensions
    let installed = list_installed_extensions(lang);
    let has_darkreader = installed.iter().any(|e| {
        e.name.to_lowercase().contains("dark reader")
            || e.id.contains("eimadpbcbfnmbkopoojfekhnkhdbieeh")
    });
    let has_bitwarden = installed.iter().any(|e| {
        e.name.to_lowercase().contains("bitwarden")
            || e.id.contains("nngceckbapebfimnlniiiahkandclblb")
    });
    let has_violentmonkey = installed.iter().any(|e| {
        e.name.to_lowercase().contains("violentmonkey") || e.id.contains("violentmonkey")
    });

    ui_card(ui, palette, |ui| {
        ui_section_title(ui, palette, t(GuiText::PopularExtensions, lang));
        ui.add_space(8.0);

        popular_ext_row(
            ui,
            palette,
            "Dark Reader",
            "Mode sombre pour les sites web",
            has_darkreader,
            lang,
            || install_chrome_extension("eimadpbcbfnmbkopoojfekhnkhdbieeh", lang),
            ext_install_status,
        );
        ui.separator();
        popular_ext_row(
            ui,
            palette,
            "Bitwarden",
            "Gestionnaire de mots de passe",
            has_bitwarden,
            lang,
            || install_chrome_extension("nngceckbapebfimnlniiiahkandclblb", lang),
            ext_install_status,
        );
        ui.separator();
        popular_ext_row(
            ui,
            palette,
            "Violentmonkey",
            "Gestionnaire de scripts utilisateur",
            has_violentmonkey,
            lang,
            || install_firefox_extension("violentmonkey", lang),
            ext_install_status,
        );
    });

    ui.add_space(12.0);

    // Installed extensions list
    ui_section_title(
        ui,
        palette,
        &format!(
            "{} ({})",
            t(GuiText::InstalledExtensionsCount, lang),
            installed.len()
        ),
    );
    ui.add_space(8.0);

    if installed.is_empty() {
        ui_card(ui, palette, |ui| {
            ui.label(
                egui::RichText::new(t(GuiText::NoExtensions, lang))
                    .color(palette.text_secondary),
            );
        });
    } else {
        for ext in installed {
            ui_card(ui, palette, |ui| {
                ui.horizontal(|ui| {
                    let (badge_text, badge_bg, badge_fg) = match ext.kind {
                        ExtensionKind::Theme => (
                            t(GuiText::ThemeBadge, lang),
                            palette.accent_soft,
                            palette.accent,
                        ),
                        ExtensionKind::Userscript => (
                            t(GuiText::UserscriptBadge, lang),
                            palette.input_bg,
                            palette.warning,
                        ),
                        ExtensionKind::ContentScript | ExtensionKind::WebExtension => (
                            t(GuiText::WebExtensionBadge, lang),
                            palette.input_bg,
                            palette.text_secondary,
                        ),
                    };
                    ui.label(
                        egui::RichText::new(&ext.name)
                            .size(14.5)
                            .color(palette.text_primary)
                            .strong(),
                    );
                    ui_badge(ui, badge_text, badge_bg, badge_fg);
                    if let Some(ref ver) = ext.version {
                        ui.label(
                            egui::RichText::new(format!("v{ver}"))
                                .size(12.0)
                                .color(palette.text_secondary),
                        );
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .add(
                                egui::Button::new(
                                    egui::RichText::new(t(GuiText::Delete, lang))
                                        .color(palette.danger),
                                )
                                .corner_radius(CornerRadius::same(6)),
                            )
                            .clicked()
                        {
                            if settings.active_theme_extension.as_deref() == Some(ext.id.as_str())
                            {
                                settings.active_theme_extension = None;
                                *settings_changed = true;
                            }
                            let _ = delete_extension(&ext.path);
                        }

                        let toggle_label = if ext.enabled {
                            t(GuiText::Disable, lang)
                        } else {
                            t(GuiText::Enable, lang)
                        };
                        if ui.button(toggle_label).clicked() {
                            set_extension_enabled(&ext.id, !ext.enabled);
                        }

                        if ext.kind == ExtensionKind::Theme && ext.enabled {
                            let is_active_theme = settings.active_theme_extension.as_deref()
                                == Some(ext.id.as_str());
                            let label = if is_active_theme {
                                t(GuiText::ResetTheme, lang)
                            } else {
                                t(GuiText::ApplyTheme, lang)
                            };
                            if ui.button(label).clicked() {
                                settings.active_theme_extension = if is_active_theme {
                                    None
                                } else {
                                    Some(ext.id.clone())
                                };
                                *settings_changed = true;
                            }
                        }
                    });
                });
                if let Some(ref desc) = ext.description {
                    ui.add_space(4.0);
                    ui.label(
                        egui::RichText::new(desc)
                            .size(12.5)
                            .color(palette.text_secondary),
                    );
                }
            });
            ui.add_space(8.0);
        }
    }

    ui.add_space(8.0);

    // Catalogs footer
    ui_card(ui, palette, |ui| {
        ui.horizontal_wrapped(|ui| {
            for (label, url) in [
                (
                    t(GuiText::OpenChromeStore, lang),
                    "https://chromewebstore.google.com/",
                ),
                (
                    t(GuiText::OpenFirefoxAddons, lang),
                    "https://addons.mozilla.org/",
                ),
                (t(GuiText::OpenGreasyFork, lang), "https://greasyfork.org/"),
            ] {
                if ui.button(label).clicked() {
                    window.queue_user_interface_command(UserInterfaceCommand::Go(url.into()));
                    *close = true;
                }
            }
            if let Some(dir) = extensions_directory_path()
                && ui.button(t(GuiText::OpenExtensionsDir, lang)).clicked()
            {
                #[cfg(target_os = "linux")]
                let _ = std::process::Command::new("xdg-open").arg(dir).spawn();
            }
        });
    });
}

#[allow(clippy::too_many_arguments)]
fn popular_ext_row(
    ui: &mut egui::Ui,
    palette: &ThemePalette,
    title: &str,
    desc: &str,
    installed: bool,
    lang: &str,
    install_fn: impl FnOnce() -> Result<String, String>,
    status_out: &mut Option<(bool, String)>,
) {
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new(title)
                .color(palette.text_primary)
                .strong(),
        );
        ui.label(
            egui::RichText::new(desc)
                .size(12.5)
                .color(palette.text_secondary),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if installed {
                ui.colored_label(palette.success, t(GuiText::InstalledBadge, lang));
            } else if ui.button("+ Installer").clicked() {
                match install_fn() {
                    Ok(msg) => *status_out = Some((true, msg)),
                    Err(e) => *status_out = Some((false, e)),
                }
            }
        });
    });
}

fn render_history(
    ui: &mut egui::Ui,
    palette: &ThemePalette,
    history: &mut Vec<String>,
    lang: &str,
    window: &ServoShellWindow,
    close: &mut bool,
    history_changed: &mut bool,
) {
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new(t(GuiText::History, lang))
                .size(22.0)
                .color(palette.text_primary)
                .strong(),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if !history.is_empty() && ui.button(t(GuiText::ClearHistory, lang)).clicked() {
                history.clear();
                *history_changed = true;
            }
        });
    });
    ui.add_space(12.0);

    if history.is_empty() {
        ui_card(ui, palette, |ui| {
            ui.label(
                egui::RichText::new(t(GuiText::NoHistory, lang))
                    .color(palette.text_secondary),
            );
        });
        return;
    }

    for url in history.clone() {
        ui_card(ui, palette, |ui| {
            ui.horizontal(|ui| {
                if ui
                    .add(egui::Button::new(&url).frame(false))
                    .clicked()
                {
                    window.queue_user_interface_command(UserInterfaceCommand::Go(url));
                    *close = true;
                }
            });
        });
        ui.add_space(6.0);
    }
}

fn render_bookmarks(
    ui: &mut egui::Ui,
    palette: &ThemePalette,
    bookmarks: &mut HashSet<String>,
    lang: &str,
    window: &ServoShellWindow,
    close: &mut bool,
    bookmarks_changed: &mut bool,
) {
    ui.label(
        egui::RichText::new(t(GuiText::Bookmarks, lang))
            .size(22.0)
            .color(palette.text_primary)
            .strong(),
    );
    ui.add_space(12.0);

    if bookmarks.is_empty() {
        ui_card(ui, palette, |ui| {
            ui.label(
                egui::RichText::new(t(GuiText::NoBookmarks, lang))
                    .color(palette.text_secondary),
            );
        });
        return;
    }

    let mut sorted: Vec<String> = bookmarks.iter().cloned().collect();
    sorted.sort_unstable();
    for url in sorted {
        ui_card(ui, palette, |ui| {
            ui.horizontal(|ui| {
                if ui
                    .add(egui::Button::new(&url).frame(false))
                    .clicked()
                {
                    window.queue_user_interface_command(UserInterfaceCommand::Go(url.clone()));
                    *close = true;
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .button(
                            egui::RichText::new(t(GuiText::Delete, lang)).color(palette.danger),
                        )
                        .clicked()
                    {
                        bookmarks.remove(&url);
                        *bookmarks_changed = true;
                    }
                });
            });
        });
        ui.add_space(6.0);
    }
}

fn ui_card(ui: &mut egui::Ui, palette: &ThemePalette, add_contents: impl FnOnce(&mut egui::Ui)) {
    egui::Frame::NONE
        .fill(palette.card_bg)
        .stroke(Stroke::new(1.0_f32, palette.card_border))
        .corner_radius(CornerRadius::same(10))
        .inner_margin(egui::Margin::same(16))
        .show(ui, add_contents);
}

fn ui_section_title(ui: &mut egui::Ui, palette: &ThemePalette, title: &str) {
    ui.label(
        egui::RichText::new(title)
            .size(15.5)
            .color(palette.text_primary)
            .strong(),
    );
}

fn ui_badge(ui: &mut egui::Ui, text: &str, bg: Color32, fg: Color32) {
    egui::Frame::NONE
        .fill(bg)
        .corner_radius(CornerRadius::same(6))
        .inner_margin(egui::Margin::symmetric(8, 3))
        .show(ui, |ui| {
            ui.label(egui::RichText::new(text).size(11.5).color(fg).strong());
        });
}

fn ui_toggle(
    ui: &mut egui::Ui,
    palette: &ThemePalette,
    value: &mut bool,
    label: &str,
) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        let desired_size = Vec2::new(38.0, 20.0);
        let (rect, response) = ui.allocate_exact_size(desired_size, Sense::click());
        if response.clicked() {
            *value = !*value;
            changed = true;
        }
        let bg = if *value {
            palette.accent
        } else {
            palette.input_border
        };
        let radius = rect.height() * 0.5;
        ui.painter().rect_filled(rect, radius, bg);
        let knob_x = if *value {
            rect.right() - radius
        } else {
            rect.left() + radius
        };
        ui.painter().circle_filled(
            egui::pos2(knob_x, rect.center().y),
            radius - 3.0,
            Color32::WHITE,
        );
        ui.add_space(6.0);
        if ui
            .add(egui::Label::new(egui::RichText::new(label).color(palette.text_primary)).sense(Sense::click()))
            .clicked()
        {
            *value = !*value;
            changed = true;
        }
    });
    changed
}
