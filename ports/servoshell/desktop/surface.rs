/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::HashSet;

use egui::{Color32, CornerRadius, Id, Key, Order, Rect, Sense, Stroke, pos2, vec2};
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
    let lang_str = settings.language.clone();
    let lang = lang_str.as_str();
    #[allow(deprecated)]
    let surface_rect = ctx.available_rect();

    if ctx.input(|i| i.key_pressed(Key::Escape)) {
        close = true;
    }

    egui::Area::new(Id::new("librenav-browser-surface"))
        .order(Order::Foreground)
        .fixed_pos(surface_rect.min)
        .show(ctx, |ui| {
            ui.set_min_size(surface_rect.size());
            ui.set_max_size(surface_rect.size());
            egui::Frame::NONE
                .fill(palette.surface_bg)
                .inner_margin(egui::Margin::symmetric(28, 22))
                .show(ui, |ui| {
                    let inner_size = surface_rect.size() - vec2(56.0, 44.0);
                    ui.set_min_size(inner_size);
                    ui.set_max_size(inner_size);

                    render_header(ui, palette, active_surface, lang, &mut close);
                    ui.add_space(14.0);
                    ui_divider(ui, palette);
                    ui.add_space(18.0);

                    ui.horizontal_top(|ui| {
                        render_sidebar(
                            ui,
                            palette,
                            active_surface,
                            lang,
                            &mut requested_surface,
                        );
                        ui.add_space(28.0);

                        let content_width = ui.available_width().min(860.0).max(320.0);
                        ui.vertical(|ui| {
                            ui.set_width(content_width);
                            egui::ScrollArea::vertical()
                                .auto_shrink([false, false])
                                .show(ui, |ui| {
                                    ui.set_width((content_width - 14.0).max(280.0));
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
                                    ui.add_space(24.0);
                                });
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
        if ui_icon_pill(ui, palette, "←", t(GuiText::BackToBrowser, lang)) {
            *close = true;
        }
        ui.add_space(12.0);
        let surface_title = match active_surface {
            BrowserSurface::Settings => t(GuiText::Settings, lang),
            BrowserSurface::History => t(GuiText::History, lang),
            BrowserSurface::Bookmarks => t(GuiText::Bookmarks, lang),
            BrowserSurface::Extensions => t(GuiText::Extensions, lang),
            BrowserSurface::None => "",
        };
        ui.label(
            egui::RichText::new("LibreNav")
                .size(17.0)
                .color(palette.text_secondary),
        );
        ui.label(
            egui::RichText::new("›")
                .size(17.0)
                .color(palette.text_secondary),
        );
        ui.label(
            egui::RichText::new(surface_title)
                .size(17.0)
                .color(palette.text_primary)
                .strong(),
        );

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui_icon_pill(ui, palette, "✕", t(GuiText::Close, lang)) {
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
    ui.vertical(|ui| {
        ui.set_width(205.0);
        let items = [
            (BrowserSurface::Settings, t(GuiText::Settings, lang)),
            (BrowserSurface::Extensions, t(GuiText::Extensions, lang)),
            (BrowserSurface::History, t(GuiText::History, lang)),
            (BrowserSurface::Bookmarks, t(GuiText::Bookmarks, lang)),
        ];
        for (surf, label) in items {
            let is_active = active_surface == surf;
            let (rect, resp) = ui.allocate_exact_size(vec2(205.0, 38.0), Sense::click());
            let bg = if is_active {
                palette.accent_soft
            } else if resp.hovered() {
                palette.tab_hover_bg
            } else {
                Color32::TRANSPARENT
            };
            ui.painter()
                .rect_filled(rect, CornerRadius::same(8), bg);

            if is_active {
                let indicator = Rect::from_min_size(
                    pos2(rect.left() + 2.0, rect.top() + 8.0),
                    vec2(3.0, rect.height() - 16.0),
                );
                ui.painter()
                    .rect_filled(indicator, CornerRadius::same(2), palette.accent);
            }

            let text_color = if is_active {
                palette.accent
            } else {
                palette.text_primary
            };
            ui.painter().text(
                pos2(rect.left() + 16.0, rect.center().y),
                egui::Align2::LEFT_CENTER,
                label,
                egui::FontId::proportional(14.5),
                text_color,
            );

            if resp.clicked() {
                *requested_surface = surf;
            }
            ui.add_space(4.0);
        }

        ui.add_space(20.0);
        ui_divider(ui, palette);
        ui.add_space(10.0);
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
    ui_page_heading(
        ui,
        palette,
        t(GuiText::Settings, lang),
        t(GuiText::PrivacyDesc, lang),
    );
    ui.add_space(16.0);

    // Language Card (Segmented Control)
    ui_card(ui, palette, |ui| {
        ui_section_title(ui, palette, t(GuiText::Language, lang));
        ui.add_space(10.0);

        egui::Frame::NONE
            .fill(palette.input_bg)
            .corner_radius(CornerRadius::same(10))
            .inner_margin(egui::Margin::same(4))
            .show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    for (code, label) in [
                        ("fr", "Français"),
                        ("en", "English"),
                        ("eo", "Esperanto"),
                        ("es", "Español"),
                    ] {
                        let is_active = settings.language == code;
                        if ui_segmented_tab(ui, palette, label, is_active) {
                            settings.language = code.into();
                            *settings_changed = true;
                            if let Some(wv) = window.active_webview()
                                && let Some(u) = wv.url()
                                && u.scheme() == "servo"
                                && u.path() == "newtab"
                                && let Ok(newtab_url) =
                                    Url::parse(&format!("servo:newtab?lang={code}"))
                            {
                                wv.load(newtab_url);
                            }
                        }
                    }
                });
            });
    });

    ui.add_space(14.0);

    // Appearance & Theme Card
    ui_card(ui, palette, |ui| {
        ui_section_title(ui, palette, t(GuiText::Appearance, lang));
        ui.add_space(4.0);
        ui.label(
            egui::RichText::new(t(GuiText::ThemeLabel, lang))
                .size(13.0)
                .color(palette.text_secondary),
        );
        ui.add_space(12.0);

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
                if ui_theme_swatch_card(ui, palette, &preview, label, is_active) {
                    settings.theme_preset = preset;
                    settings.active_theme_extension = None;
                    *settings_changed = true;
                }
                ui.add_space(8.0);
            }
        });

        let installed = list_installed_extensions(lang);
        let theme_exts: Vec<&ExtensionInfo> = installed
            .iter()
            .filter(|e| e.kind == ExtensionKind::Theme && e.enabled)
            .collect();
        if !theme_exts.is_empty() {
            ui.add_space(12.0);
            ui_divider(ui, palette);
            ui.add_space(10.0);
            for ext in theme_exts {
                let is_ext_active =
                    settings.active_theme_extension.as_deref() == Some(ext.id.as_str());
                ui.horizontal(|ui| {
                    ui_badge(
                        ui,
                        t(GuiText::ThemeBadge, lang),
                        palette.accent_soft,
                        palette.accent,
                    );
                    ui.label(
                        egui::RichText::new(&ext.name)
                            .color(palette.text_primary)
                            .strong(),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if is_ext_active {
                            if ui_secondary_button(ui, palette, t(GuiText::ResetTheme, lang)) {
                                settings.active_theme_extension = None;
                                *settings_changed = true;
                            }
                        } else if ui_primary_button(ui, palette, t(GuiText::ApplyTheme, lang)) {
                            settings.active_theme_extension = Some(ext.id.clone());
                            *settings_changed = true;
                        }
                    });
                });
            }
        }

        ui.add_space(14.0);
        ui_divider(ui, palette);
        ui.add_space(10.0);

        *settings_changed |= ui_toggle_row(
            ui,
            palette,
            &mut settings.show_home_button,
            t(GuiText::ShowHomeButton, lang),
            None,
        );
        ui.add_space(8.0);
        ui_divider(ui, palette);
        ui.add_space(8.0);
        *settings_changed |= ui_toggle_row(
            ui,
            palette,
            &mut settings.show_bookmarks_bar,
            t(GuiText::ShowBookmarksBar, lang),
            None,
        );
        ui.add_space(8.0);
        ui_divider(ui, palette);
        ui.add_space(8.0);
        *settings_changed |= ui_toggle_row(
            ui,
            palette,
            &mut settings.content_scripts_enabled,
            t(GuiText::ContentScriptsLabel, lang),
            Some(t(GuiText::ContentScriptsDesc, lang)),
        );
    });

    ui.add_space(14.0);

    // Homepage & Search Engine Card
    ui_card(ui, palette, |ui| {
        ui_section_title(ui, palette, t(GuiText::Homepage, lang));
        ui.label(
            egui::RichText::new(t(GuiText::HomepageDesc, lang))
                .size(12.5)
                .color(palette.text_secondary),
        );
        ui.add_space(6.0);
        *settings_changed |= ui_text_input(ui, palette, &mut settings.homepage, "");
        if !settings.homepage_is_valid() {
            ui.add_space(4.0);
            ui.colored_label(palette.danger, t(GuiText::InvalidHomepage, lang));
        }

        ui.add_space(14.0);
        ui_divider(ui, palette);
        ui.add_space(12.0);

        ui_section_title(ui, palette, t(GuiText::SearchEngine, lang));
        ui.label(
            egui::RichText::new(t(GuiText::SearchEngineDesc, lang))
                .size(12.5)
                .color(palette.text_secondary),
        );
        ui.add_space(6.0);
        *settings_changed |= ui_text_input(ui, palette, &mut settings.searchpage, "");
        if !settings.searchpage_is_valid() {
            ui.add_space(4.0);
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
    ui_page_heading(
        ui,
        palette,
        t(GuiText::ExtensionsTitle, lang),
        t(GuiText::ExtensionsSubtitle, lang),
    );
    ui.add_space(16.0);

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
                if ui_secondary_button(ui, palette, "Dashboard") {
                    window.queue_user_interface_command(UserInterfaceCommand::Go(
                        "servo:ublock-dashboard".into(),
                    ));
                    *close = true;
                }
                if ui_primary_button(ui, palette, t(GuiText::UblockOpenPopup, lang)) {
                    window.queue_user_interface_command(UserInterfaceCommand::Go(
                        "servo:ublock".into(),
                    ));
                    *close = true;
                }
            });
        });
        ui.add_space(10.0);
        ui_divider(ui, palette);
        ui.add_space(8.0);
        *settings_changed |= ui_toggle_row(
            ui,
            palette,
            &mut settings.ublock_enabled,
            t(GuiText::UblockPowerOn, lang),
            None,
        );
    });

    ui.add_space(14.0);

    // Install card
    ui_card(ui, palette, |ui| {
        ui_section_title(ui, palette, t(GuiText::InstallExtension, lang));
        ui.add_space(8.0);
        ui_text_input(
            ui,
            palette,
            ext_install_input,
            t(GuiText::ExtensionInputPlaceholder, lang),
        );
        ui.add_space(10.0);
        ui.horizontal_wrapped(|ui| {
            if ui_primary_button(ui, palette, t(GuiText::InstallFromChromeStore, lang)) {
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
            if ui_secondary_button(ui, palette, t(GuiText::InstallFromFirefoxAddons, lang)) {
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
            if ui_secondary_button(ui, palette, t(GuiText::InstallLocalFile, lang)) {
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
            ui.add_space(8.0);
            let color = if *success {
                palette.success
            } else {
                palette.danger
            };
            let prefix = if *success { "✓" } else { "✕" };
            ui.colored_label(color, format!("{prefix} {msg}"));
        }
    });

    ui.add_space(14.0);

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
        ui.add_space(10.0);

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
        ui.add_space(8.0);
        ui_divider(ui, palette);
        ui.add_space(8.0);
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
        ui.add_space(8.0);
        ui_divider(ui, palette);
        ui.add_space(8.0);
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

    ui.add_space(14.0);

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
                        if ui_danger_button(ui, palette, t(GuiText::Delete, lang)) {
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
                        if ui_secondary_button(ui, palette, toggle_label) {
                            set_extension_enabled(&ext.id, !ext.enabled);
                        }

                        if ext.kind == ExtensionKind::Theme && ext.enabled {
                            let is_active_theme = settings.active_theme_extension.as_deref()
                                == Some(ext.id.as_str());
                            let action = if is_active_theme {
                                ui_secondary_button(ui, palette, t(GuiText::ResetTheme, lang))
                            } else {
                                ui_primary_button(ui, palette, t(GuiText::ApplyTheme, lang))
                            };
                            if action {
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
                if ui_secondary_button(ui, palette, label) {
                    window.queue_user_interface_command(UserInterfaceCommand::Go(url.into()));
                    *close = true;
                }
            }
            if let Some(dir) = extensions_directory_path()
                && ui_secondary_button(ui, palette, t(GuiText::OpenExtensionsDir, lang))
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
        ui.vertical(|ui| {
            ui.label(
                egui::RichText::new(title)
                    .size(14.0)
                    .color(palette.text_primary)
                    .strong(),
            );
            ui.label(
                egui::RichText::new(desc)
                    .size(12.5)
                    .color(palette.text_secondary),
            );
        });
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if installed {
                ui_badge(
                    ui,
                    t(GuiText::InstalledBadge, lang),
                    palette.accent_soft,
                    palette.success,
                );
            } else if ui_primary_button(ui, palette, "+ Installer") {
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
            if !history.is_empty()
                && ui_danger_button(ui, palette, t(GuiText::ClearHistory, lang))
            {
                history.clear();
                *history_changed = true;
            }
        });
    });
    ui.add_space(14.0);

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
            if ui_link_row(ui, palette, &url) {
                window.queue_user_interface_command(UserInterfaceCommand::Go(url));
                *close = true;
            }
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
    ui_page_heading(ui, palette, t(GuiText::Bookmarks, lang), "");
    ui.add_space(14.0);

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
                if ui_link_row(ui, palette, &url) {
                    window.queue_user_interface_command(UserInterfaceCommand::Go(url.clone()));
                    *close = true;
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui_danger_button(ui, palette, t(GuiText::Delete, lang)) {
                        bookmarks.remove(&url);
                        *bookmarks_changed = true;
                    }
                });
            });
        });
        ui.add_space(6.0);
    }
}

fn ui_page_heading(ui: &mut egui::Ui, palette: &ThemePalette, title: &str, subtitle: &str) {
    ui.label(
        egui::RichText::new(title)
            .size(22.0)
            .color(palette.text_primary)
            .strong(),
    );
    if !subtitle.is_empty() {
        ui.add_space(2.0);
        ui.label(
            egui::RichText::new(subtitle)
                .size(13.0)
                .color(palette.text_secondary),
        );
    }
}

fn ui_divider(ui: &mut egui::Ui, palette: &ThemePalette) {
    let width = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(vec2(width, 1.0), Sense::hover());
    ui.painter().rect_filled(rect, CornerRadius::ZERO, palette.separator);
}

fn ui_card(ui: &mut egui::Ui, palette: &ThemePalette, add_contents: impl FnOnce(&mut egui::Ui)) {
    let card_width = ui.available_width();
    egui::Frame::NONE
        .fill(palette.card_bg)
        .stroke(Stroke::new(1.0_f32, palette.card_border))
        .corner_radius(CornerRadius::same(12))
        .inner_margin(egui::Margin::same(18))
        .show(ui, |ui| {
            ui.set_width((card_width - 36.0).max(200.0));
            ui.vertical(add_contents);
        });
}

fn ui_section_title(ui: &mut egui::Ui, palette: &ThemePalette, title: &str) {
    ui.label(
        egui::RichText::new(title)
            .size(15.5)
            .color(palette.text_primary)
            .strong(),
    );
}

fn ui_icon_pill(ui: &mut egui::Ui, palette: &ThemePalette, glyph: &str, tooltip: &str) -> bool {
    let (rect, resp) = ui.allocate_exact_size(vec2(34.0, 34.0), Sense::click());
    let bg = if resp.hovered() {
        palette.tab_hover_bg
    } else {
        palette.card_bg
    };
    ui.painter().rect_filled(rect, CornerRadius::same(17), bg);
    ui.painter().rect_stroke(
        rect,
        CornerRadius::same(17),
        Stroke::new(1.0_f32, palette.card_border),
        egui::StrokeKind::Inside,
    );
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        glyph,
        egui::FontId::proportional(15.0),
        palette.text_primary,
    );
    resp.on_hover_text(tooltip).clicked()
}

fn ui_segmented_tab(ui: &mut egui::Ui, palette: &ThemePalette, label: &str, active: bool) -> bool {
    let btn = egui::Button::new(
        egui::RichText::new(label)
            .size(13.5)
            .color(if active {
                palette.text_on_accent
            } else {
                palette.text_secondary
            })
            .strong(),
    )
    .fill(if active {
        palette.accent
    } else {
        Color32::TRANSPARENT
    })
    .stroke(Stroke::NONE)
    .corner_radius(CornerRadius::same(7));
    ui.add(btn).clicked()
}

fn ui_primary_button(ui: &mut egui::Ui, palette: &ThemePalette, label: &str) -> bool {
    let btn = egui::Button::new(
        egui::RichText::new(label)
            .size(13.0)
            .color(palette.text_on_accent)
            .strong(),
    )
    .fill(palette.accent)
    .stroke(Stroke::NONE)
    .corner_radius(CornerRadius::same(8));
    ui.add(btn).clicked()
}

fn ui_secondary_button(ui: &mut egui::Ui, palette: &ThemePalette, label: &str) -> bool {
    let btn = egui::Button::new(
        egui::RichText::new(label)
            .size(13.0)
            .color(palette.text_primary),
    )
    .fill(palette.tab_hover_bg)
    .stroke(Stroke::NONE)
    .corner_radius(CornerRadius::same(8));
    ui.add(btn).clicked()
}

fn ui_danger_button(ui: &mut egui::Ui, palette: &ThemePalette, label: &str) -> bool {
    let btn = egui::Button::new(
        egui::RichText::new(label)
            .size(13.0)
            .color(palette.danger),
    )
    .fill(Color32::from_rgba_unmultiplied(
        palette.danger.r(),
        palette.danger.g(),
        palette.danger.b(),
        26,
    ))
    .stroke(Stroke::NONE)
    .corner_radius(CornerRadius::same(8));
    ui.add(btn).clicked()
}

fn ui_link_row(ui: &mut egui::Ui, palette: &ThemePalette, url: &str) -> bool {
    let resp = ui.add(
        egui::Label::new(
            egui::RichText::new(url)
                .size(13.5)
                .color(palette.text_primary),
        )
        .sense(Sense::click()),
    );
    resp.on_hover_cursor(egui::CursorIcon::PointingHand)
        .clicked()
}

fn ui_text_input(
    ui: &mut egui::Ui,
    palette: &ThemePalette,
    value: &mut String,
    hint: &str,
) -> bool {
    let mut changed = false;
    egui::Frame::NONE
        .fill(palette.input_bg)
        .stroke(Stroke::new(1.0_f32, palette.input_border))
        .corner_radius(CornerRadius::same(8))
        .inner_margin(egui::Margin::symmetric(12, 8))
        .show(ui, |ui| {
            let mut edit = egui::TextEdit::singleline(value)
                .frame(egui::Frame::NONE)
                .desired_width(f32::INFINITY);
            if !hint.is_empty() {
                edit = edit.hint_text(hint);
            }
            changed = ui.add(edit).changed();
        });
    changed
}

fn ui_theme_swatch_card(
    ui: &mut egui::Ui,
    host_palette: &ThemePalette,
    preview: &ThemePalette,
    label: &str,
    active: bool,
) -> bool {
    let size = vec2(150.0, 74.0);
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click());

    let border = if active {
        Stroke::new(2.0_f32, host_palette.accent)
    } else if resp.hovered() {
        Stroke::new(1.5_f32, host_palette.text_secondary)
    } else {
        Stroke::new(1.0_f32, host_palette.card_border)
    };

    ui.painter()
        .rect_filled(rect, CornerRadius::same(10), preview.surface_bg);

    // Mini window chrome bar at top of preview card
    let top_bar = Rect::from_min_size(rect.min, vec2(rect.width(), 22.0));
    ui.painter().rect_filled(
        top_bar,
        CornerRadius {
            nw: 10,
            ne: 10,
            sw: 0,
            se: 0,
        },
        preview.tab_strip_bg,
    );
    // Mini active tab pill + accent dot
    let mini_tab = Rect::from_min_size(pos2(rect.left() + 8.0, rect.top() + 5.0), vec2(46.0, 14.0));
    ui.painter()
        .rect_filled(mini_tab, CornerRadius::same(4), preview.toolbar_bg);
    ui.painter().circle_filled(
        pos2(rect.right() - 14.0, rect.top() + 11.0),
        4.5,
        preview.accent,
    );

    // Theme label centered in bottom area
    ui.painter().text(
        pos2(rect.center().x, rect.top() + 48.0),
        egui::Align2::CENTER_CENTER,
        label,
        egui::FontId::proportional(13.0),
        preview.text_primary,
    );

    ui.painter()
        .rect_stroke(rect, CornerRadius::same(10), border, egui::StrokeKind::Inside);

    resp.on_hover_cursor(egui::CursorIcon::PointingHand)
        .clicked()
}

fn ui_badge(ui: &mut egui::Ui, text: &str, bg: Color32, fg: Color32) {
    egui::Frame::NONE
        .fill(bg)
        .corner_radius(CornerRadius::same(12))
        .inner_margin(egui::Margin::symmetric(10, 3))
        .show(ui, |ui| {
            ui.label(egui::RichText::new(text).size(11.5).color(fg).strong());
        });
}

fn ui_toggle_row(
    ui: &mut egui::Ui,
    palette: &ThemePalette,
    value: &mut bool,
    label: &str,
    description: Option<&str>,
) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.label(
                egui::RichText::new(label)
                    .size(13.5)
                    .color(palette.text_primary),
            );
            if let Some(desc) = description {
                ui.label(
                    egui::RichText::new(desc)
                        .size(12.0)
                        .color(palette.text_secondary),
                );
            }
        });
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let desired_size = vec2(42.0, 22.0);
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
                pos2(knob_x, rect.center().y),
                radius - 3.0,
                Color32::WHITE,
            );
        });
    });
    changed
}
