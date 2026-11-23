/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::{HashMap, HashSet};
#[cfg(any(
    target_os = "windows",
    target_os = "linux",
    target_os = "freebsd",
    target_os = "macos"
))]
use std::fs;
#[cfg(any(
    target_os = "windows",
    target_os = "linux",
    target_os = "freebsd",
    target_os = "macos"
))]
use std::path::Path;
use std::rc::Rc;
use std::sync::Arc;

use accesskit::Affine;
use dpi::PhysicalSize;
use egui::text::{CCursor, CCursorRange};
use egui::text_edit::TextEditState;
use egui::{
    Button, Color32, FontDefinitions, Id, Key, Label, LayerId, Modifiers, Order, PaintCallback,
    Panel, Sense, Stroke, Vec2, WidgetInfo, WidgetType, pos2,
};
#[cfg(any(
    target_os = "windows",
    target_os = "linux",
    target_os = "freebsd",
    target_os = "macos"
))]
use egui::{FontData, FontFamily};
use egui_glow::{CallbackFn, EguiGlow};
use egui_winit::EventResponse;
use euclid::{Length, Point2D, Rect, Scale, Size2D};
#[cfg(any(
    target_os = "windows",
    target_os = "linux",
    target_os = "freebsd",
    target_os = "macos"
))]
use log::info;
use log::warn;
use servo::{
    DeviceIndependentPixel, DevicePixel, Image, LoadStatus, OffscreenRenderingContext, PixelFormat,
    RenderingContext, WebView, WebViewId,
};
use url::Url;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoopProxy};
use winit::window::Window;

use crate::desktop::event_loop::AppEvent;
use crate::desktop::extensions::{
    ExtensionKind, blocked_requests_count, list_installed_extensions, sync_adblock_settings,
};
use crate::desktop::headed_window;
pub(crate) use crate::desktop::settings::BrowserSurface;
use crate::desktop::settings::{
    BrowserSettings, GuiText, LIBRENAV_HISTORY_FILE, LIBRENAV_SETTINGS_FILE,
    load_bookmarks, load_json_profile, save_bookmarks, save_json_profile, t,
};
use crate::desktop::theme::ThemePalette;
use crate::running_app_state::{RunningAppState, UserInterfaceCommand};
use crate::window::{ServoShellWindow, TopLevelWebViewCreationRequest};

// Chromium 155 desktop metrics recovered from the matching upstream tag for the
// locally installed 155.0.8059.39 artifact. Keeping these values together makes
// the reconstruction explicit and prevents visual tuning from drifting.
const CHROMIUM_TAB_STRIP_HEIGHT: f32 = 41.0;
const CHROMIUM_TAB_HEIGHT: f32 = 35.0;
const CHROMIUM_TAB_WIDTH: f32 = 256.0;
const CHROMIUM_TAB_TOP_RADIUS: u8 = 10;
const CHROMIUM_TOOLBAR_HEIGHT: f32 = 52.0;
const CHROMIUM_TOOLBAR_BUTTON_SIZE: f32 = 34.0;
const CHROMIUM_LOCATION_BAR_HEIGHT: f32 = 34.0;

#[derive(Clone, Copy)]
enum ChromeIcon {
    Back,
    Forward,
    Reload,
    Stop,
    Home,
    Bookmark { filled: bool },
    Menu,
    NewTab,
    Close,
    Profile,
    Extensions,
    Shield,
}

/// The user interface of a headed servoshell. Currently this is implemented via
/// egui.
pub struct Gui {
    rendering_context: Rc<OffscreenRenderingContext>,
    context: EguiGlow,
    toolbar_height: Length<f32, DeviceIndependentPixel>,

    location: String,

    /// Whether the location has been edited by the user without clicking Go.
    location_dirty: bool,

    /// The [`LoadStatus`] of the active `WebView`.
    load_status: LoadStatus,

    /// The text to display in the status bar on the bottom of the window.
    status_text: Option<String>,

    /// Whether or not the current `WebView` can navigate backward.
    can_go_back: bool,

    /// Whether or not the current `WebView` can navigate forward.
    can_go_forward: bool,

    /// Bookmarks stored in LibreNav's local profile directory.
    bookmarked_urls: HashSet<String>,

    /// Recently visited URLs, newest first, stored in LibreNav's local profile.
    history: Vec<String>,

    /// Browser-shell preferences that are independent from Servo engine preferences.
    browser_settings: BrowserSettings,

    /// The currently open browser-management surface.
    browser_surface: BrowserSurface,

    /// Handle to the GPU texture of the favicon.
    ///
    /// These need to be cached across egui draw calls.
    favicon_textures: HashMap<WebViewId, (egui::TextureHandle, egui::load::SizedTexture)>,

    /// AccessKit tree updates pending the next egui tick.
    /// This allows us to ensure that graft nodes are sent before the subtrees they graft.
    pending_accesskit_updates: Vec<accesskit::TreeUpdate>,

    /// Text input buffer for installing extensions from URL or local file path.
    ext_install_input: String,

    /// Last status message from extension installation (is_success, message).
    ext_install_status: Option<(bool, String)>,
}

fn truncate_with_ellipsis(input: &str, max_length: usize) -> String {
    if input.chars().count() > max_length {
        let truncated: String = input.chars().take(max_length.saturating_sub(1)).collect();
        format!("{}…", truncated)
    } else {
        input.to_string()
    }
}

#[cfg(any(
    target_os = "windows",
    target_os = "linux",
    target_os = "freebsd",
    target_os = "macos"
))]
fn load_cjk_fonts(font_candidates: &[(&str, &str)]) -> FontDefinitions {
    let mut fonts = FontDefinitions::default();
    let mut loaded_font_names = Vec::new();

    for (path_str, font_name) in font_candidates.iter() {
        let font_path = Path::new(path_str);
        if font_path.exists() {
            match fs::read(font_path) {
                Ok(bytes) => {
                    if !fonts.font_data.contains_key(*font_name) {
                        fonts
                            .font_data
                            .insert(font_name.to_string(), Arc::new(FontData::from_owned(bytes)));
                        loaded_font_names.push(font_name.to_string());
                        info!("Loaded font: {}", font_name);
                    }
                },
                Err(error) => {
                    info!("Failed to read font {}: {}", font_name, error);
                },
            }
        }
    }

    if !loaded_font_names.is_empty() {
        let proportional = fonts.families.get_mut(&FontFamily::Proportional).unwrap();
        for font_name in loaded_font_names.iter() {
            proportional.insert(0, font_name.clone());
        }
    }

    fonts
}

#[cfg(target_os = "windows")]
fn configure_fonts() -> FontDefinitions {
    load_cjk_fonts(&[
        (r"C:\Windows\Fonts\malgun.ttf", "Malgun Gothic"), // Korean
        (r"C:\Windows\Fonts\msyh.ttc", "Microsoft YaHei"), // Chinese + Japanese
    ])
}

#[cfg(any(target_os = "linux", target_os = "freebsd"))]
fn configure_fonts() -> FontDefinitions {
    load_cjk_fonts(&[
        (
            "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
            "Noto Sans CJK",
        ), // Ubuntu/Debian
        (
            "/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc",
            "Noto Sans CJK",
        ), // Fedora/Arch
        // FreeBSD splits the Noto CJK fonts into regional subsets
        (
            "/usr/local/share/fonts/noto/NotoSansCJKhk-Regular.otf",
            "Noto Sans CJK HK",
        ),
        (
            "/usr/local/share/fonts/noto/NotoSansCJKjp-Regular.otf",
            "Noto Sans CJK JP",
        ),
        (
            "/usr/local/share/fonts/noto/NotoSansCJKkr-Regular.otf",
            "Noto Sans CJK KR",
        ),
        (
            "/usr/local/share/fonts/noto/NotoSansCJKsc-Regular.otf",
            "Noto Sans CJK SC",
        ),
        (
            "/usr/local/share/fonts/noto/NotoSansCJKtc-Regular.otf",
            "Noto Sans CJK TC",
        ),
        (
            "/usr/share/fonts/truetype/wqy/wqy-microhei.ttc",
            "WenQuanYi Micro Hei",
        ), // common fallback
        (
            "/usr/local/share/fonts/wqy/wqy-microhei.ttc",
            "WenQuanYi Micro Hei",
        ), // FreeBSD
    ])
}

#[cfg(target_os = "macos")]
fn configure_fonts() -> FontDefinitions {
    load_cjk_fonts(&[
        (
            "/System/Library/Fonts/AppleSDGothicNeo.ttc",
            "Apple SD Gothic Neo",
        ), // Korean
        ("/System/Library/Fonts/STHeiti Medium.ttc", "STHeiti Medium"), // Chinese + Japanese
    ])
}

impl Drop for Gui {
    fn drop(&mut self) {
        self.rendering_context
            .make_current()
            .expect("Could not make window RenderingContext current");
        self.context.destroy();
    }
}

impl Gui {
    pub(crate) fn new(
        winit_window: &Window,
        event_loop: &ActiveEventLoop,
        event_loop_proxy: EventLoopProxy<AppEvent>,
        rendering_context: Rc<OffscreenRenderingContext>,
        initial_url: Url,
    ) -> Self {
        rendering_context
            .make_current()
            .expect("Could not make window RenderingContext current");
        let mut context = EguiGlow::new(
            event_loop,
            rendering_context.glow_gl_api(),
            None,
            None,
            false,
        );

        let font_definitions = configure_fonts();
        context.egui_ctx.set_fonts(font_definitions);

        context
            .egui_winit
            .init_accesskit(event_loop, winit_window, event_loop_proxy);
        winit_window.set_visible(true);

        context.egui_ctx.options_mut(|options| {
            // Disable the builtin egui handlers for the Ctrl+Plus, Ctrl+Minus and Ctrl+0
            // shortcuts as they don't work well with servoshell's `device-pixel-ratio` CLI argument.
            options.zoom_with_keyboard = false;

            // On platforms where winit fails to obtain a system theme, fall back to a light theme
            // since it is the more common default.
            options.fallback_theme = egui::Theme::Light;
        });

        let browser_settings: BrowserSettings = load_json_profile(LIBRENAV_SETTINGS_FILE);
        sync_adblock_settings(&browser_settings);
        let palette = Self::compute_palette(&browser_settings);
        palette.apply_to_egui(&context.egui_ctx);

        Self {
            rendering_context,
            context,
            toolbar_height: Default::default(),
            location: initial_url.to_string(),
            location_dirty: false,
            load_status: LoadStatus::Complete,
            status_text: None,
            can_go_back: false,
            can_go_forward: false,
            bookmarked_urls: load_bookmarks(),
            history: load_json_profile(LIBRENAV_HISTORY_FILE),
            browser_settings,
            browser_surface: match std::env::var("LIBRENAV_START_SURFACE").as_deref() {
                Ok("settings") => BrowserSurface::Settings,
                Ok("history") => BrowserSurface::History,
                Ok("bookmarks") => BrowserSurface::Bookmarks,
                Ok("extensions") => BrowserSurface::Extensions,
                _ => BrowserSurface::None,
            },
            favicon_textures: Default::default(),
            pending_accesskit_updates: vec![],
            ext_install_input: String::new(),
            ext_install_status: None,
        }
    }

    fn compute_palette(settings: &BrowserSettings) -> ThemePalette {
        let base = ThemePalette::from_preset(&settings.theme_preset);
        if let Some(ref ext_id) = settings.active_theme_extension {
            let extensions = list_installed_extensions(&settings.language);
            if let Some(ext) = extensions.into_iter().find(|e| e.id == *ext_id && e.enabled)
                && let Some(ref theme_colors) = ext.theme_colors
            {
                return base.with_extension_theme(theme_colors);
            }
        }
        base
    }

    pub(crate) fn has_keyboard_focus(&self) -> bool {
        self.context
            .egui_ctx
            .memory(|memory| memory.focused().is_some())
    }

    pub(crate) fn surrender_focus(&self) {
        self.context.egui_ctx.memory_mut(|memory| {
            if let Some(focused) = memory.focused() {
                memory.surrender_focus(focused);
            }
        });
    }

    pub(crate) fn on_window_event(
        &mut self,
        winit_window: &Window,
        event: &WindowEvent,
    ) -> EventResponse {
        self.context.on_window_event(winit_window, event)
    }

    /// The height of the top toolbar of this user inteface ie the distance from the top of the
    /// window to the position of the `WebView`.
    pub(crate) fn toolbar_height(&self) -> Length<f32, DeviceIndependentPixel> {
        self.toolbar_height
    }

    pub(crate) fn has_open_surface(&self) -> bool {
        self.browser_surface != BrowserSurface::None
    }

    pub(crate) fn wants_pointer_input(&self) -> bool {
        self.context.egui_ctx.egui_wants_pointer_input()
    }

    pub(crate) fn is_pointer_over_area(&self) -> bool {
        self.context.egui_ctx.is_pointer_over_egui()
    }

    pub(crate) fn any_popup_open(&self) -> bool {
        egui::Popup::is_any_open(&self.context.egui_ctx)
    }

    pub(crate) fn is_in_egui_toolbar_rect(
        &self,
        position: Point2D<f32, DeviceIndependentPixel>,
    ) -> bool {
        position.y < self.toolbar_height.get()
    }

    pub(crate) fn open_surface(&mut self, surface: BrowserSurface) {
        self.browser_surface = surface;
    }

    pub(crate) fn toggle_bookmark(&mut self, url: &str) {
        if self.bookmarked_urls.contains(url) {
            self.bookmarked_urls.remove(url);
        } else {
            self.bookmarked_urls.insert(url.to_string());
        }
        save_bookmarks(&self.bookmarked_urls);
    }

    pub(crate) fn toggle_bookmarks_bar(&mut self) {
        self.browser_settings.show_bookmarks_bar = !self.browser_settings.show_bookmarks_bar;
        save_json_profile(LIBRENAV_SETTINGS_FILE, &self.browser_settings);
    }

    /// Draw a native vector icon button with Chromium-like interaction states.
    fn icon_button(
        ui: &mut egui::Ui,
        icon: ChromeIcon,
        accessible_label: &str,
        size: f32,
    ) -> egui::Response {
        let (rect, response) = ui.allocate_exact_size(Vec2::splat(size), Sense::click());
        response.widget_info(|| {
            let mut info = WidgetInfo::new(WidgetType::Button);
            info.label = Some(accessible_label.into());
            info
        });

        if ui.is_rect_visible(rect) {
            let painter = ui.painter_at(rect);
            if response.is_pointer_button_down_on() {
                painter.circle_filled(rect.center(), size * 0.47, Color32::from_rgb(74, 76, 80));
            } else if response.hovered() {
                painter.circle_filled(rect.center(), size * 0.47, Color32::from_rgb(60, 64, 67));
            }
            if response.has_focus() {
                painter.circle_stroke(
                    rect.center(),
                    size * 0.47,
                    Stroke::new(1.5_f32, Color32::from_rgb(129, 201, 255)),
                );
            }

            let color = if ui.is_enabled() {
                Color32::from_rgb(232, 234, 237)
            } else {
                Color32::from_rgb(128, 134, 139)
            };
            let stroke = Stroke::new(1.8_f32, color);
            let center = rect.center();
            let radius = size * 0.22;
            let point = |x: f32, y: f32| center + egui::vec2(x, y);

            match icon {
                ChromeIcon::Back | ChromeIcon::Forward => {
                    let direction = if matches!(icon, ChromeIcon::Back) {
                        1.0
                    } else {
                        -1.0
                    };
                    painter.line_segment(
                        [point(5.0 * direction, -6.0), point(-direction, 0.0)],
                        stroke,
                    );
                    painter.line_segment(
                        [point(-direction, 0.0), point(5.0 * direction, 6.0)],
                        stroke,
                    );
                    painter.line_segment(
                        [point(-direction, 0.0), point(-7.0 * direction, 0.0)],
                        stroke,
                    );
                },
                ChromeIcon::Reload => {
                    let points = (0..18)
                        .map(|index| {
                            let angle = 0.35 + index as f32 * 5.0 / 17.0;
                            point(radius * angle.cos(), radius * angle.sin())
                        })
                        .collect();
                    painter.add(egui::Shape::line(points, stroke));
                    painter.line_segment([point(5.8, -7.0), point(7.8, -1.5)], stroke);
                    painter.line_segment([point(7.8, -1.5), point(2.2, -1.3)], stroke);
                },
                ChromeIcon::Stop => {
                    painter.line_segment([point(-5.0, -5.0), point(5.0, 5.0)], stroke);
                    painter.line_segment([point(5.0, -5.0), point(-5.0, 5.0)], stroke);
                },
                ChromeIcon::Home => {
                    painter.line_segment([point(-7.0, 0.0), point(0.0, -6.5)], stroke);
                    painter.line_segment([point(0.0, -6.5), point(7.0, 0.0)], stroke);
                    painter.line_segment([point(-5.0, -1.5), point(-5.0, 6.0)], stroke);
                    painter.line_segment([point(-5.0, 6.0), point(5.0, 6.0)], stroke);
                    painter.line_segment([point(5.0, 6.0), point(5.0, -1.5)], stroke);
                },
                ChromeIcon::Bookmark { filled } => {
                    let points: Vec<_> = (0..10)
                        .map(|index| {
                            let angle = -std::f32::consts::FRAC_PI_2
                                + index as f32 * std::f32::consts::PI / 5.0;
                            let star_radius = if index % 2 == 0 { 7.0 } else { 3.2 };
                            point(star_radius * angle.cos(), star_radius * angle.sin())
                        })
                        .collect();
                    if filled {
                        painter.add(egui::Shape::convex_polygon(points, color, Stroke::NONE));
                    } else {
                        painter.add(egui::Shape::closed_line(points, stroke));
                    }
                },
                ChromeIcon::Menu => {
                    for offset in [-5.0, 0.0, 5.0] {
                        painter.circle_filled(point(0.0, offset), 1.5, color);
                    }
                },
                ChromeIcon::NewTab => {
                    painter.line_segment([point(-6.0, 0.0), point(6.0, 0.0)], stroke);
                    painter.line_segment([point(0.0, -6.0), point(0.0, 6.0)], stroke);
                },
                ChromeIcon::Close => {
                    painter.line_segment([point(-4.5, -4.5), point(4.5, 4.5)], stroke);
                    painter.line_segment([point(4.5, -4.5), point(-4.5, 4.5)], stroke);
                },
                ChromeIcon::Profile => {
                    painter.circle_stroke(point(0.0, -3.5), 3.5, stroke);
                    let arc_points: Vec<_> = (0..11)
                        .map(|i| {
                            let angle = std::f32::consts::PI * 0.15
                                + i as f32 * std::f32::consts::PI * 0.7 / 10.0;
                            point(6.5 * angle.cos(), 7.0 - 5.5 * angle.sin())
                        })
                        .collect();
                    painter.add(egui::Shape::line(arc_points, stroke));
                },
                ChromeIcon::Extensions => {
                    // Smooth vector puzzle-piece silhouette
                    let mut pts = Vec::new();
                    // Left edge -> top-left corner
                    pts.push(point(-5.5, 4.5));
                    pts.push(point(-5.5, -4.5));
                    pts.push(point(-1.8, -4.5));
                    // Top knob arc
                    for i in 0..=8 {
                        let angle = std::f32::consts::PI + (i as f32 / 8.0) * std::f32::consts::PI;
                        pts.push(point(2.0 * angle.cos(), -4.8 + 2.2 * angle.sin()));
                    }
                    // Top-right corner
                    pts.push(point(1.8, -4.5));
                    pts.push(point(4.5, -4.5));
                    pts.push(point(4.5, -1.8));
                    // Right knob arc
                    for i in 0..=8 {
                        let angle = -std::f32::consts::FRAC_PI_2
                            + (i as f32 / 8.0) * std::f32::consts::PI;
                        pts.push(point(4.8 + 2.2 * angle.cos(), 2.0 * angle.sin()));
                    }
                    // Bottom-right corner -> bottom-left corner
                    pts.push(point(4.5, 1.8));
                    pts.push(point(4.5, 4.5));
                    pts.push(point(-5.5, 4.5));
                    painter.add(egui::Shape::closed_line(pts, stroke));
                },
                ChromeIcon::Shield => {
                    let pts = vec![
                        point(0.0, -6.5),
                        point(5.5, -4.5),
                        point(5.5, 1.0),
                        point(0.0, 6.5),
                        point(-5.5, 1.0),
                        point(-5.5, -4.5),
                    ];
                    painter.add(egui::Shape::closed_line(pts, stroke));
                },
            }
        }

        response
    }

    /// Draws a browser tab, checking for clicks and queues appropriate [`UserInterfaceCommand`]s.
    /// Using a custom widget here would've been nice, but it doesn't seem as though egui
    /// supports that, so we arrange multiple Widgets in a way that they look connected.
    fn browser_tab(
        ui: &mut egui::Ui,
        window: &ServoShellWindow,
        webview: WebView,
        favicon_texture: Option<egui::load::SizedTexture>,
    ) {
        let label = match (webview.page_title(), webview.url()) {
            (Some(title), _) if !title.is_empty() => title,
            (_, Some(url)) => url.to_string(),
            _ => "New Tab".into(),
        };

        let inactive_bg_color = Color32::from_rgb(32, 33, 36);
        let active_bg_color = Color32::from_rgb(53, 54, 58);
        let active = window.active_webview().map(|webview| webview.id()) == Some(webview.id());

        // Setup a tab frame that will contain the favicon, title and close button
        let mut tab_frame = egui::Frame::NONE
            .corner_radius(CHROMIUM_TAB_TOP_RADIUS)
            .inner_margin(egui::Margin::symmetric(8, 6))
            .begin(ui);
        {
            tab_frame.content_ui.set_min_height(CHROMIUM_TAB_HEIGHT);
            tab_frame.content_ui.set_max_width(CHROMIUM_TAB_WIDTH);
            let visuals = tab_frame.content_ui.visuals_mut();
            // Remove the stroke so we don't see the border between the close button and the label
            visuals.widgets.active.bg_stroke.width = 0.0;
            visuals.widgets.hovered.bg_stroke.width = 0.0;
            // Now we make sure the fill color is always the same, irrespective of state, that way
            // we can make sure that both the label and close button have the same background color
            visuals.widgets.noninteractive.weak_bg_fill = inactive_bg_color;
            visuals.widgets.inactive.weak_bg_fill = inactive_bg_color;
            visuals.widgets.hovered.weak_bg_fill = active_bg_color;
            visuals.widgets.active.weak_bg_fill = active_bg_color;
            visuals.selection.bg_fill = active_bg_color;
            visuals.selection.stroke.color = visuals.widgets.active.fg_stroke.color;
            visuals.widgets.hovered.fg_stroke.color = visuals.widgets.active.fg_stroke.color;

            // Expansion would also show that they are 2 separate widgets
            visuals.widgets.active.expansion = 0.0;
            visuals.widgets.hovered.expansion = 0.0;

            if let Some(favicon) = favicon_texture {
                tab_frame.content_ui.add(
                    egui::Image::from_texture(favicon)
                        .fit_to_exact_size(egui::vec2(16.0, 16.0))
                        .bg_fill(egui::Color32::TRANSPARENT),
                );
            }

            let tab = tab_frame
                .content_ui
                .add(Button::selectable(
                    active,
                    truncate_with_ellipsis(&label, 24),
                ))
                .on_hover_ui(|ui| {
                    ui.label(&label);
                });

            let close_button = Self::icon_button(
                &mut tab_frame.content_ui,
                ChromeIcon::Close,
                "Fermi langeton",
                24.0,
            );
            if close_button.clicked() || close_button.middle_clicked() || tab.middle_clicked() {
                window
                    .queue_user_interface_command(UserInterfaceCommand::CloseWebView(webview.id()));
            } else if !active && tab.clicked() {
                window.activate_webview(webview.id());
            }
        }

        let response = tab_frame.allocate_space(ui);
        let fill_color = if active || response.hovered() {
            active_bg_color
        } else {
            inactive_bg_color
        };
        tab_frame.frame.fill = fill_color;
        tab_frame.end(ui);
    }

    pub(crate) fn update(
        &mut self,
        state: &RunningAppState,
        window: &ServoShellWindow,
        headed_window: &headed_window::HeadedWindow,
    ) {
        self.rendering_context
            .make_current()
            .expect("Could not make RenderingContext current");
        let Self {
            rendering_context,
            context,
            toolbar_height,
            location,
            location_dirty,
            favicon_textures,
            bookmarked_urls,
            history,
            browser_settings,
            browser_surface,
            ext_install_input,
            ext_install_status,
            ..
        } = self;

        let winit_window = headed_window.winit_window();
        let lang_str = browser_settings.language.clone();
        let lang = lang_str.as_str();
        let palette = Self::compute_palette(browser_settings);
        context.run(winit_window, |ctx| {
            load_pending_favicons(ctx, window, favicon_textures);

            // TODO: While in fullscreen add some way to mitigate the increased phishing risk
            // when not displaying the URL bar: https://github.com/servo/servo/issues/32443
            // Show toolbar unless fullscreen is from document (web API)
            if !headed_window.is_fullscreen_from_document() {
                // Chromium-style ordering: tabs at the top, navigation controls below.
                let tab_frame = egui::Frame::default()
                    .fill(palette.tab_strip_bg)
                    .inner_margin(egui::Margin::symmetric(8, 5));
                Panel::top("tabs").frame(tab_frame).show_inside(ctx, |ui| {
                    ui.set_min_height(CHROMIUM_TAB_STRIP_HEIGHT);
                    egui::ScrollArea::horizontal()
                        .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
                        .show(ui, |ui| {
                            ui.allocate_ui_with_layout(
                                ui.available_size(),
                                egui::Layout::left_to_right(egui::Align::Center),
                                |ui| {
                                    for (id, webview) in window.webviews() {
                                        let favicon = favicon_textures
                                            .get(&id)
                                            .map(|(_, favicon)| favicon)
                                            .copied();
                                        Self::browser_tab(ui, window, webview, favicon);
                                    }

                                    let new_tab_button = Gui::icon_button(
                                        ui,
                                        ChromeIcon::NewTab,
                                        t(GuiText::NewTab, lang),
                                        CHROMIUM_TOOLBAR_BUTTON_SIZE,
                                    )
                                    .on_hover_text(t(GuiText::NewTab, lang));
                                    if new_tab_button.clicked() {
                                        window.queue_user_interface_command(
                                            UserInterfaceCommand::NewWebView,
                                        );
                                    }
                                },
                            );
                        });
                });

                let toolbar_frame = egui::Frame::default()
                    .fill(palette.toolbar_bg)
                    .inner_margin(egui::Margin::symmetric(8, 6));
                let outer = Panel::top("toolbar")
                    .frame(toolbar_frame)
                    .show_inside(ctx, |ui| {
                        ui.set_min_height(CHROMIUM_TOOLBAR_HEIGHT);
                        ui.allocate_ui_with_layout(
                            ui.available_size(),
                            egui::Layout::left_to_right(egui::Align::Center),
                            |ui| {
                                let back_button = ui
                                    .add_enabled_ui(self.can_go_back, |ui| {
                                        Gui::icon_button(
                                            ui,
                                            ChromeIcon::Back,
                                            t(GuiText::Back, lang),
                                            CHROMIUM_TOOLBAR_BUTTON_SIZE,
                                        )
                                        .on_hover_text(t(GuiText::Back, lang))
                                    })
                                    .inner;
                                if back_button.clicked() {
                                    *location_dirty = false;
                                    window.queue_user_interface_command(UserInterfaceCommand::Back);
                                }

                                let forward_button = ui
                                    .add_enabled_ui(self.can_go_forward, |ui| {
                                        Gui::icon_button(
                                            ui,
                                            ChromeIcon::Forward,
                                            t(GuiText::Forward, lang),
                                            CHROMIUM_TOOLBAR_BUTTON_SIZE,
                                        )
                                        .on_hover_text(t(GuiText::Forward, lang))
                                    })
                                    .inner;
                                if forward_button.clicked() {
                                    *location_dirty = false;
                                    window.queue_user_interface_command(
                                        UserInterfaceCommand::Forward,
                                    );
                                }

                                match self.load_status {
                                    LoadStatus::Started | LoadStatus::HeadParsed => {
                                        let stop_button = Gui::icon_button(
                                            ui,
                                            ChromeIcon::Stop,
                                            t(GuiText::Stop, lang),
                                            CHROMIUM_TOOLBAR_BUTTON_SIZE,
                                        )
                                        .on_hover_text(t(GuiText::Stop, lang));
                                        if stop_button.clicked() {
                                            warn!("Do not support stop yet.");
                                        }
                                    },
                                    LoadStatus::Complete => {
                                        let reload_button = Gui::icon_button(
                                            ui,
                                            ChromeIcon::Reload,
                                            t(GuiText::Reload, lang),
                                            CHROMIUM_TOOLBAR_BUTTON_SIZE,
                                        )
                                        .on_hover_text(t(GuiText::Reload, lang));
                                        if reload_button.clicked() {
                                            *location_dirty = false;
                                            window.queue_user_interface_command(
                                                UserInterfaceCommand::Reload,
                                            );
                                        }
                                    },
                                }

                                if browser_settings.show_home_button {
                                    let home_button = Gui::icon_button(
                                        ui,
                                        ChromeIcon::Home,
                                        t(GuiText::Home, lang),
                                        CHROMIUM_TOOLBAR_BUTTON_SIZE,
                                    )
                                    .on_hover_text(t(GuiText::Home, lang));
                                    if home_button.clicked() {
                                        *location_dirty = false;
                                        let homepage = if browser_settings.homepage_is_valid() {
                                            browser_settings.homepage.clone()
                                        } else {
                                            "servo:newtab".into()
                                        };
                                        window.queue_user_interface_command(
                                            UserInterfaceCommand::Go(homepage),
                                        );
                                    }
                                }

                                let active_url = window
                                    .active_webview()
                                    .and_then(|webview| webview.url())
                                    .map(|url| url.to_string());

                                ui.allocate_ui_with_layout(
                                    ui.available_size(),
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        let menu_button = Gui::icon_button(
                                            ui,
                                            ChromeIcon::Menu,
                                            t(GuiText::MainMenu, lang),
                                            CHROMIUM_TOOLBAR_BUTTON_SIZE,
                                        )
                                        .on_hover_text(t(GuiText::MainMenu, lang));
                                        let profile_button = Gui::icon_button(
                                            ui,
                                            ChromeIcon::Profile,
                                            t(GuiText::Profile, lang),
                                            CHROMIUM_TOOLBAR_BUTTON_SIZE,
                                        )
                                        .on_hover_text(t(GuiText::Profile, lang));
                                        let extensions_button = Gui::icon_button(
                                            ui,
                                            ChromeIcon::Extensions,
                                            t(GuiText::Extensions, lang),
                                            CHROMIUM_TOOLBAR_BUTTON_SIZE,
                                        )
                                        .on_hover_text(t(GuiText::Extensions, lang));
                                        let ublock_button = Gui::icon_button(
                                            ui,
                                            ChromeIcon::Shield,
                                            "uBlock Origin",
                                            CHROMIUM_TOOLBAR_BUTTON_SIZE,
                                        )
                                        .on_hover_text("uBlock Origin");

                                        egui::Popup::menu(&ublock_button).show(|ui| {
                                            ui.set_min_width(300.0);
                                            ui.vertical(|ui| {
                                                ui.horizontal(|ui| {
                                                    ui.heading("uBlock Origin");
                                                    ui.weak("v1.75.0");
                                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                        if browser_settings.ublock_enabled {
                                                            ui.colored_label(Color32::from_rgb(52, 168, 83), t(GuiText::ActiveStatus, lang));
                                                        } else {
                                                            ui.colored_label(Color32::from_rgb(234, 67, 53), "✕ Off");
                                                        }
                                                    });
                                                });
                                                ui.add_space(4.0);
                                                ui.separator();
                                                ui.add_space(6.0);

                                                let current_host = active_url.as_ref()
                                                    .and_then(|u| url::Url::parse(u).ok())
                                                    .and_then(|u| u.host_str().map(|h| h.to_string()));

                                                let host_display = current_host.as_deref().unwrap_or("local");
                                                let is_host_exempt = current_host.as_ref()
                                                    .is_some_and(|h| browser_settings.ublock_disabled_hosts.contains(h));
                                                let is_filtering = browser_settings.ublock_enabled && !is_host_exempt;

                                                ui.vertical_centered(|ui| {
                                                    ui.strong(format!("Site : {host_display}"));
                                                    ui.add_space(6.0);

                                                    let (power_color, power_icon_color) = if is_filtering {
                                                        (Color32::from_rgb(26, 115, 232), Color32::WHITE)
                                                    } else {
                                                        (Color32::from_rgb(95, 99, 104), Color32::from_rgb(218, 220, 224))
                                                    };

                                                    let (power_rect, power_resp) = ui.allocate_exact_size(Vec2::splat(56.0), Sense::click());
                                                    let p_center = power_rect.center();
                                                    ui.painter().circle_filled(p_center, 28.0, power_color);
                                                    let power_stroke = Stroke::new(2.8_f32, power_icon_color);
                                                    // Vector power symbol: open arc + vertical stem
                                                    let arc_pts: Vec<egui::Pos2> = (0..=24)
                                                        .map(|i| {
                                                            let a = -std::f32::consts::FRAC_PI_2 + 0.55
                                                                + (i as f32 / 24.0) * (std::f32::consts::TAU - 1.1);
                                                            p_center + egui::vec2(12.5 * a.cos(), 12.5 * a.sin())
                                                        })
                                                        .collect();
                                                    ui.painter().add(egui::Shape::line(arc_pts, power_stroke));
                                                    ui.painter().line_segment(
                                                        [p_center + egui::vec2(0.0, -14.0), p_center + egui::vec2(0.0, -2.0)],
                                                        power_stroke,
                                                    );

                                                    if power_resp.on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                                                        if let Some(ref h) = current_host {
                                                            if is_host_exempt {
                                                                browser_settings.ublock_disabled_hosts.remove(h);
                                                            } else {
                                                                browser_settings.ublock_disabled_hosts.insert(h.clone());
                                                            }
                                                        } else {
                                                            browser_settings.ublock_enabled = !browser_settings.ublock_enabled;
                                                        }
                                                        sync_adblock_settings(browser_settings);
                                                        save_json_profile(LIBRENAV_SETTINGS_FILE, browser_settings);
                                                    }

                                                    ui.add_space(4.0);
                                                    if is_filtering {
                                                        ui.colored_label(palette.success, t(GuiText::UblockPowerOn, lang));
                                                    } else {
                                                        ui.colored_label(palette.danger, t(GuiText::UblockPowerOff, lang));
                                                    }
                                                    ui.label(
                                                        egui::RichText::new(format!(
                                                            "{} : {}",
                                                            t(GuiText::BlockedRequestsLabel, lang),
                                                            blocked_requests_count()
                                                        ))
                                                        .size(12.0)
                                                        .color(palette.text_secondary),
                                                    );
                                                });

                                                ui.add_space(8.0);
                                                ui.separator();
                                                ui.add_space(8.0);

                                                ui.label(
                                                    egui::RichText::new("Interfaces officielles uBlock :")
                                                        .strong()
                                                        .size(13.0)
                                                );
                                                ui.add_space(4.0);

                                                for (label, target_url) in [
                                                    ("Popup officiel uBlock Origin", "servo:ublock"),
                                                    ("Tableau de bord & filtres uBlock", "servo:ublock-dashboard"),
                                                    ("Filtres tiers (3p-filters.html)", "resource:///ublock/3p-filters.html"),
                                                ] {
                                                    let btn = egui::Button::new(
                                                        egui::RichText::new(label)
                                                            .size(12.5)
                                                            .color(palette.text_primary),
                                                    )
                                                    .fill(palette.tab_hover_bg)
                                                    .stroke(Stroke::NONE)
                                                    .corner_radius(egui::CornerRadius::same(8));
                                                    if ui.add_sized([ui.available_width(), 28.0], btn).clicked() {
                                                        window.queue_user_interface_command(UserInterfaceCommand::Go(target_url.into()));
                                                        ui.close();
                                                    }
                                                    ui.add_space(4.0);
                                                }

                                                ui.add_space(4.0);
                                                ui.separator();
                                                ui.add_space(6.0);

                                                let manage_btn = egui::Button::new(
                                                    egui::RichText::new(t(GuiText::ManageExtensions, lang))
                                                        .size(13.0)
                                                        .color(palette.text_on_accent)
                                                        .strong(),
                                                )
                                                .fill(palette.accent)
                                                .stroke(Stroke::NONE)
                                                .corner_radius(egui::CornerRadius::same(8));
                                                if ui.add_sized([ui.available_width(), 30.0], manage_btn).clicked() {
                                                    *browser_surface = BrowserSurface::Extensions;
                                                    ui.close();
                                                }
                                            });
                                        });

                                        egui::Popup::menu(&extensions_button).show(|ui| {
                                            ui.set_min_width(290.0);
                                            ui.vertical(|ui| {
                                                ui.horizontal(|ui| {
                                                    ui.label(
                                                        egui::RichText::new(t(GuiText::Extensions, lang))
                                                            .size(15.5)
                                                            .color(palette.text_primary)
                                                            .strong(),
                                                    );
                                                });
                                                ui.label(
                                                    egui::RichText::new(t(GuiText::ExtensionsSubtitle, lang))
                                                        .size(12.0)
                                                        .color(palette.text_secondary),
                                                );
                                                ui.add_space(8.0);
                                                ui.separator();
                                                ui.add_space(8.0);

                                                let installed = list_installed_extensions(lang);
                                                if installed.is_empty() {
                                                    ui.label(
                                                        egui::RichText::new(t(GuiText::NoExtensions, lang))
                                                            .size(12.5)
                                                            .color(palette.text_secondary),
                                                    );
                                                } else {
                                                    for ext in installed {
                                                        ui.horizontal(|ui| {
                                                            let badge = match ext.kind {
                                                                ExtensionKind::Theme => t(GuiText::ThemeBadge, lang),
                                                                ExtensionKind::Userscript => t(GuiText::UserscriptBadge, lang),
                                                                ExtensionKind::ContentScript | ExtensionKind::WebExtension => {
                                                                    t(GuiText::WebExtensionBadge, lang)
                                                                },
                                                            };
                                                            ui.label(
                                                                egui::RichText::new(&ext.name)
                                                                    .size(13.0)
                                                                    .color(palette.text_primary)
                                                                    .strong(),
                                                            );
                                                            ui.label(
                                                                egui::RichText::new(badge)
                                                                    .size(11.0)
                                                                    .color(palette.text_secondary),
                                                            );
                                                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                                let (dot_color, label, text_color) = if ext.enabled {
                                                                    (palette.success, t(GuiText::ActiveStatus, lang), palette.success)
                                                                } else {
                                                                    (palette.text_secondary, t(GuiText::InactiveStatus, lang), palette.text_secondary)
                                                                };
                                                                ui.colored_label(text_color, label);
                                                                let (dot_rect, _) = ui.allocate_exact_size(egui::vec2(8.0, 8.0), Sense::hover());
                                                                ui.painter().circle_filled(dot_rect.center(), 3.5, dot_color);
                                                            });
                                                        });
                                                        ui.add_space(3.0);
                                                    }
                                                }

                                                ui.add_space(8.0);
                                                ui.separator();
                                                ui.add_space(8.0);

                                                let manage_btn = egui::Button::new(
                                                    egui::RichText::new(t(GuiText::ManageExtensions, lang))
                                                        .size(13.0)
                                                        .color(palette.text_on_accent)
                                                        .strong(),
                                                )
                                                .fill(palette.accent)
                                                .stroke(Stroke::NONE)
                                                .corner_radius(egui::CornerRadius::same(8));
                                                if ui.add_sized([ui.available_width(), 32.0], manage_btn).clicked() {
                                                    *browser_surface = BrowserSurface::Extensions;
                                                    ui.close();
                                                }
                                                ui.add_space(6.0);
                                                for (label, url) in [
                                                    (t(GuiText::OpenChromeStore, lang), "https://chromewebstore.google.com/"),
                                                    (t(GuiText::OpenFirefoxAddons, lang), "https://addons.mozilla.org/"),
                                                ] {
                                                    let btn = egui::Button::new(
                                                        egui::RichText::new(label)
                                                            .size(12.5)
                                                            .color(palette.text_primary),
                                                    )
                                                    .fill(palette.tab_hover_bg)
                                                    .stroke(Stroke::NONE)
                                                    .corner_radius(egui::CornerRadius::same(8));
                                                    if ui.add_sized([ui.available_width(), 28.0], btn).clicked() {
                                                        window.queue_user_interface_command(UserInterfaceCommand::Go(url.into()));
                                                        ui.close();
                                                    }
                                                    ui.add_space(4.0);
                                                }
                                            });
                                        });

                                        egui::Popup::menu(&profile_button).show(|ui| {
                                            ui.set_min_width(260.0);
                                            ui.vertical(|ui| {
                                                ui.heading(t(GuiText::ProfileTitle, lang));
                                                ui.weak(t(GuiText::ProfileSubtitle, lang));
                                                ui.add_space(8.0);
                                                ui.separator();
                                                ui.add_space(8.0);

                                                ui.label(t(GuiText::Language, lang));
                                                ui.horizontal(|ui| {
                                                    for (code, label) in [
                                                        ("eo", "Esperanto"),
                                                        ("fr", "Français"),
                                                        ("en", "English"),
                                                        ("es", "Español"),
                                                    ] {
                                                        let is_active = browser_settings.language == code;
                                                        if ui.selectable_label(is_active, label).clicked() {
                                                            browser_settings.language = code.into();
                                                            save_json_profile(LIBRENAV_SETTINGS_FILE, browser_settings);
                                                            if let Some(wv) = window.active_webview()
                                                                && let Some(u) = wv.url()
                                                                && u.scheme() == "servo"
                                                                && u.path() == "newtab"
                                                            {
                                                                let newtab_url = Url::parse(&format!("servo:newtab?lang={code}")).unwrap();
                                                                wv.load(newtab_url);
                                                            }
                                                        }
                                                    }
                                                });
                                                ui.add_space(8.0);
                                                ui.separator();
                                                ui.add_space(8.0);

                                                if ui.button(t(GuiText::Settings, lang)).clicked() {
                                                    *browser_surface = BrowserSurface::Settings;
                                                    ui.close();
                                                }
                                                if ui.button(t(GuiText::History, lang)).clicked() {
                                                    *browser_surface = BrowserSurface::History;
                                                    ui.close();
                                                }
                                                if ui.button(t(GuiText::Bookmarks, lang)).clicked() {
                                                    *browser_surface = BrowserSurface::Bookmarks;
                                                    ui.close();
                                                }
                                                if ui.button(t(GuiText::Extensions, lang)).clicked() {
                                                    *browser_surface = BrowserSurface::Extensions;
                                                    ui.close();
                                                }
                                            });
                                        });

                                        egui::Popup::menu(&menu_button).show(|ui| {
                                            ui.set_min_width(220.0);
                                            if ui.button(t(GuiText::NewTab, lang)).clicked() {
                                                window.queue_user_interface_command(
                                                    UserInterfaceCommand::NewWebView,
                                                );
                                                ui.close();
                                            }

                                            if ui.button(t(GuiText::NewWindow, lang)).clicked() {
                                                let url = Url::parse(&format!("servo:newtab?lang={lang}")).expect(
                                                    "servo:newtab must always be a valid URL",
                                                );
                                                window.queue_user_interface_command(
                                                    UserInterfaceCommand::NewWindow(
                                                        TopLevelWebViewCreationRequest::WithUrl(
                                                            url,
                                                        ),
                                                    ),
                                                );
                                                ui.close();
                                            }

                                            if ui.button(t(GuiText::Reload, lang)).clicked() {
                                                window.queue_user_interface_command(
                                                    UserInterfaceCommand::Reload,
                                                );
                                                ui.close();
                                            }

                                            ui.separator();
                                            ui.horizontal(|ui| {
                                                ui.label(t(GuiText::Zoom, lang));
                                                if ui.button(" - ").clicked()
                                                    && let Some(wv) = window.active_webview()
                                                {
                                                    wv.set_page_zoom((wv.page_zoom() - 0.1).max(0.25));
                                                }
                                                let zoom_pct = window
                                                    .active_webview()
                                                    .map(|wv| (wv.page_zoom() * 100.0).round() as i32)
                                                    .unwrap_or(100);
                                                if ui.button(format!("{zoom_pct}%")).clicked()
                                                    && let Some(wv) = window.active_webview()
                                                {
                                                    wv.set_page_zoom(1.0);
                                                }
                                                if ui.button(" + ").clicked()
                                                    && let Some(wv) = window.active_webview()
                                                {
                                                    wv.set_page_zoom((wv.page_zoom() + 0.1).min(5.0));
                                                }
                                            });

                                            ui.separator();
                                            if ui.button(t(GuiText::History, lang)).clicked() {
                                                *browser_surface = BrowserSurface::History;
                                                ui.close();
                                            }
                                            if ui.button(t(GuiText::Bookmarks, lang)).clicked() {
                                                *browser_surface = BrowserSurface::Bookmarks;
                                                ui.close();
                                            }
                                            if ui.button(t(GuiText::Extensions, lang)).clicked() {
                                                *browser_surface = BrowserSurface::Extensions;
                                                ui.close();
                                            }
                                            if ui.button(t(GuiText::Settings, lang)).clicked() {
                                                *browser_surface = BrowserSurface::Settings;
                                                ui.close();
                                            }

                                            ui.separator();
                                            ui.menu_button(t(GuiText::Language, lang), |ui| {
                                                for (code, label) in [
                                                    ("eo", "Esperanto"),
                                                    ("fr", "Français"),
                                                    ("en", "English"),
                                                    ("es", "Español"),
                                                ] {
                                                    let is_active = browser_settings.language == code;
                                                    if ui.selectable_label(is_active, label).clicked() {
                                                        browser_settings.language = code.into();
                                                        save_json_profile(
                                                            LIBRENAV_SETTINGS_FILE,
                                                            browser_settings,
                                                        );
                                                        if let Some(wv) = window.active_webview()
                                                            && let Some(u) = wv.url()
                                                            && u.scheme() == "servo"
                                                            && u.path() == "newtab"
                                                        {
                                                            let newtab_url = Url::parse(&format!("servo:newtab?lang={code}")).unwrap();
                                                            wv.load(newtab_url);
                                                        }
                                                    }
                                                }
                                            });

                                            ui.separator();
                                            if ui
                                                .checkbox(
                                                    &mut browser_settings.show_bookmarks_bar,
                                                    t(GuiText::ShowBookmarksBar, lang),
                                                )
                                                .changed()
                                            {
                                                save_json_profile(
                                                    LIBRENAV_SETTINGS_FILE,
                                                    browser_settings,
                                                );
                                            }

                                            let mut experimental_preferences_enabled =
                                                state.experimental_preferences_enabled();
                                            if ui
                                                .checkbox(
                                                    &mut experimental_preferences_enabled,
                                                    t(GuiText::ExperimentalFeatures, lang),
                                                )
                                                .changed()
                                            {
                                                state.set_experimental_preferences_enabled(
                                                    experimental_preferences_enabled,
                                                );
                                                *location_dirty = false;
                                                window.queue_user_interface_command(
                                                    UserInterfaceCommand::ReloadAll,
                                                );
                                            }

                                            ui.separator();
                                            if ui.button(t(GuiText::Exit, lang)).clicked() {
                                                state.schedule_exit();
                                                ui.close();
                                            }
                                        });

                                        if let Some(url) = active_url.as_ref() {
                                            let is_bookmarked = bookmarked_urls.contains(url);
                                            let bookmark_label = if is_bookmarked {
                                                t(GuiText::BookmarkRemove, lang)
                                            } else {
                                                t(GuiText::BookmarkAdd, lang)
                                            };
                                            let bookmark_button = Gui::icon_button(
                                                ui,
                                                ChromeIcon::Bookmark {
                                                    filled: is_bookmarked,
                                                },
                                                bookmark_label,
                                                CHROMIUM_TOOLBAR_BUTTON_SIZE,
                                            )
                                            .on_hover_text(bookmark_label);
                                            if bookmark_button.clicked() {
                                                if is_bookmarked {
                                                    bookmarked_urls.remove(url);
                                                } else {
                                                    bookmarked_urls.insert(url.clone());
                                                }
                                                save_bookmarks(bookmarked_urls);
                                            }
                                        }

                                        let location_id = egui::Id::new("location_input");
                                        let location_frame = egui::Frame::NONE
                                            .fill(Color32::from_rgb(32, 33, 36))
                                            .corner_radius(CHROMIUM_LOCATION_BAR_HEIGHT as u8 / 2)
                                            .inner_margin(egui::Margin::symmetric(12, 8));
                                        let location_field = location_frame
                                            .show(ui, |ui| {
                                                ui.set_min_height(
                                                    CHROMIUM_LOCATION_BAR_HEIGHT - 16.0,
                                                );
                                                ui.horizontal(|ui| {
                                                    let connection_label = active_url
                                                        .as_deref()
                                                        .map(|url| {
                                                            if url.starts_with("https://") {
                                                                "https"
                                                            } else if url.starts_with("http://") {
                                                                "http"
                                                            } else {
                                                                "local"
                                                            }
                                                        })
                                                        .unwrap_or("local");
                                                    ui.weak(connection_label);
                                                    ui.add_sized(
                                                        ui.available_size(),
                                                        egui::TextEdit::singleline(location)
                                                            .id(location_id)
                                                            .frame(egui::Frame::NONE)
                                                            .hint_text(t(GuiText::SearchPlaceholder, lang)),
                                                    )
                                                })
                                                .inner
                                            })
                                            .inner;

                                        if location_field.changed() {
                                            *location_dirty = true;
                                        }
                                        // Handle adddress bar shortcut.
                                        if ui.input(|i| {
                                            if cfg!(target_os = "macos") {
                                                i.clone().consume_key(Modifiers::COMMAND, Key::L)
                                            } else {
                                                i.clone().consume_key(Modifiers::COMMAND, Key::L)
                                                    || i.clone().consume_key(Modifiers::ALT, Key::D)
                                            }
                                        }) {
                                            // The focus request immediately makes gained_focus return true.
                                            location_field.request_focus();
                                        }
                                        // Select address bar text when it's focused (click or shortcut).
                                        if location_field.gained_focus()
                                            && let Some(mut state) =
                                                TextEditState::load(ui.ctx(), location_id)
                                        {
                                            // Select the whole input.
                                            state.cursor.set_char_range(Some(CCursorRange::two(
                                                CCursor::new(0),
                                                CCursor::new(location.len()),
                                            )));
                                            state.store(ui.ctx(), location_id);
                                        }
                                        // Navigate to address when enter is pressed in the address bar.
                                        if location_field.lost_focus()
                                            && ui.input(|i| i.clone().key_pressed(Key::Enter))
                                        {
                                            let destination =
                                                crate::parser::location_bar_input_to_url(
                                                    location,
                                                    if browser_settings.searchpage_is_valid() {
                                                        &browser_settings.searchpage
                                                    } else {
                                                        "https://revsearch.revlibertaire.com/search.php?q=%s"
                                                    },
                                                )
                                                .map(|url| url.into_url().to_string())
                                                .unwrap_or_else(|| location.clone());
                                            window.queue_user_interface_command(
                                                UserInterfaceCommand::Go(destination),
                                            );
                                        }
                                    },
                                );
                            },
                        );
                    });

                if browser_settings.show_bookmarks_bar {
                    let bookmarks_frame = egui::Frame::default()
                        .fill(palette.bookmarks_bg)
                        .inner_margin(egui::Margin::symmetric(10, 4));
                    let bookmarks_panel = Panel::top("bookmarks_bar")
                        .frame(bookmarks_frame)
                        .show_inside(ctx, |ui| {
                            ui.set_min_height(28.0);
                            ui.allocate_ui_with_layout(
                                ui.available_size(),
                                egui::Layout::left_to_right(egui::Align::Center),
                                |ui| {
                                    if bookmarked_urls.is_empty() {
                                        ui.weak(t(GuiText::BookmarksBarEmpty, lang));
                                    } else {
                                        let mut sorted_bookmarks: Vec<_> =
                                            bookmarked_urls.iter().cloned().collect();
                                        sorted_bookmarks.sort_unstable();
                                        for url in sorted_bookmarks.iter().take(10) {
                                            let display_label = if let Ok(parsed) = url::Url::parse(url) {
                                                parsed.host_str().unwrap_or(url).to_string()
                                            } else {
                                                url.clone()
                                            };
                                            let short_label = truncate_with_ellipsis(&display_label, 18);
                                            let btn = ui.button(format!("• {short_label}")).on_hover_text(url);
                                            if btn.clicked() {
                                                *location_dirty = false;
                                                window.queue_user_interface_command(
                                                    UserInterfaceCommand::Go(url.clone()),
                                                );
                                            }
                                        }
                                    }
                                },
                            );
                        });
                    *toolbar_height = Length::new(bookmarks_panel.response.rect.max.y);
                } else {
                    *toolbar_height = Length::new(outer.response.rect.max.y);
                }
            } else {
                *toolbar_height = Length::default();
            }

            let settings_changed = crate::desktop::surface::show_browser_surface(
                ctx,
                &palette,
                browser_surface,
                browser_settings,
                history,
                bookmarked_urls,
                ext_install_input,
                ext_install_status,
                window,
            );
            if settings_changed {
                let updated_palette = Self::compute_palette(browser_settings);
                updated_palette.apply_to_egui(ctx);
            }

            let scale =
                Scale::<_, DeviceIndependentPixel, DevicePixel>::new(ctx.pixels_per_point());

            headed_window.for_each_active_dialog(window, |dialog| dialog.update(ctx));

            // If the top parts of the GUI changed size, then update the size of the WebView and also
            // the size of its RenderingContext.
            let available_rect = ctx.available_rect_before_wrap();

            // Build a graft node for each WebView.
            let affine = {
                // The grafted WebView tree reports bounds in device pixels relative to the
                // WebView's own origin, so this node supplies the offset of the WebView within
                // the window, and scales it to the same scale as the rest of the nodes in egui's
                // AccessKit tree.
                let scale = (1.0 / window.platform_window().hidpi_scale_factor().get()) as f64;
                let x = available_rect.min.x as f64;
                let y = available_rect.min.y as f64;
                Affine::new([scale, 0.0, 0.0, scale, x, y])
            };
            for (webview_id, webview) in window.webviews() {
                if let Some(tree_id) = webview.accesskit_tree_id() {
                    let id = egui::Id::new(webview_id);
                    ctx.accesskit_node_builder(id, |node| {
                        node.set_tree_id(tree_id);
                        // Only the transform is set: AccessKit consumers exclude graft nodes from
                        // the presented tree, so bounds on this node would never be read.
                        node.set_transform(affine);
                    });
                }
            }
            let size = Size2D::new(available_rect.width(), available_rect.height()) * scale;
            if let Some(webview) = window.active_webview()
                && size != webview.size()
            {
                // `rect` is sized to just the WebView viewport, which is required by
                // `OffscreenRenderingContext` See:
                // <https://github.com/servo/servo/issues/38369#issuecomment-3138378527>
                webview.resize(PhysicalSize::new(size.width as u32, size.height as u32))
            }

            if let Some(status_text) = &self.status_text {
                egui::Tooltip::always_open(
                    ctx.clone(),
                    LayerId::new(Order::Tooltip, Id::new("tooltip")),
                    "tooltip layer".into(),
                    pos2(0.0, available_rect.max.y),
                )
                .show(|ui| ui.add(Label::new(status_text.clone()).extend()));
            }

            window.repaint_webviews();

            if let Some(render_to_parent) = rendering_context.render_to_parent_callback() {
                ctx.layer_painter(LayerId::background()).add(PaintCallback {
                    rect: available_rect,
                    callback: Arc::new(CallbackFn::new(move |info, painter| {
                        let clip = info.viewport_in_pixels();
                        let rect_in_parent = Rect::new(
                            Point2D::new(clip.left_px, clip.from_bottom_px),
                            Size2D::new(clip.width_px, clip.height_px),
                        );
                        render_to_parent(painter.gl(), rect_in_parent)
                    })),
                });
            }
        });

        // If any egui widget requested a repaint, also request a repaint for our
        // containing window. This allows egui widget to animate on their own.
        if self.context.egui_ctx.has_requested_repaint() {
            window.set_needs_repaint();
        }

        let adapter = self
            .context
            .egui_winit
            .accesskit
            .as_mut()
            .expect("guaranteed by Gui::new()");
        for tree_update in self.pending_accesskit_updates.drain(..) {
            adapter.update_if_active(|| tree_update);
        }
    }

    /// Paint the GUI, as of the last update.
    pub(crate) fn paint(&mut self, window: &Window) {
        self.rendering_context
            .make_current()
            .expect("Could not make RenderingContext current");
        self.rendering_context
            .parent_context()
            .prepare_for_rendering();
        self.context.paint(window);
        self.rendering_context.parent_context().present();
    }

    /// Updates the location field from the given [`RunningAppState`], unless the user has started
    /// editing it without clicking Go, returning true iff it has changed (needing an egui update).
    fn update_location_in_toolbar(&mut self, window: &ServoShellWindow) -> bool {
        // User edited without clicking Go?
        if self.location_dirty {
            return false;
        }

        let current_url_string = window
            .active_webview()
            .and_then(|webview| Some(webview.url()?.to_string()));
        match current_url_string {
            Some(location) if location != self.location => {
                if (location.starts_with("https://") || location.starts_with("http://"))
                    && self.history.first() != Some(&location)
                {
                    self.history.retain(|entry| entry != &location);
                    self.history.insert(0, location.clone());
                    self.history.truncate(500);
                    save_json_profile(LIBRENAV_HISTORY_FILE, &self.history);
                }
                self.location = location;
                true
            },
            _ => false,
        }
    }

    fn update_load_status(&mut self, window: &ServoShellWindow) -> bool {
        let state_status = window
            .active_webview()
            .map(|webview| webview.load_status())
            .unwrap_or(LoadStatus::Complete);
        let old_status = std::mem::replace(&mut self.load_status, state_status);
        let status_changed = old_status != self.load_status;

        // When the load status changes, we want the new changes to the URL to start
        // being reflected in the location bar.
        if status_changed {
            self.location_dirty = false;
        }

        status_changed
    }

    fn update_status_text(&mut self, window: &ServoShellWindow) -> bool {
        let state_status = window
            .active_webview()
            .and_then(|webview| webview.status_text());
        let old_status = std::mem::replace(&mut self.status_text, state_status);
        old_status != self.status_text
    }

    fn update_can_go_back_and_forward(&mut self, window: &ServoShellWindow) -> bool {
        let (can_go_back, can_go_forward) = window
            .active_webview()
            .map(|webview| (webview.can_go_back(), webview.can_go_forward()))
            .unwrap_or((false, false));
        let old_can_go_back = std::mem::replace(&mut self.can_go_back, can_go_back);
        let old_can_go_forward = std::mem::replace(&mut self.can_go_forward, can_go_forward);
        old_can_go_back != self.can_go_back || old_can_go_forward != self.can_go_forward
    }

    /// Updates all fields taken from the given [`ServoShellWindow`], such as the location field.
    /// Returns true iff the egui needs an update.
    pub(crate) fn update_webview_data(&mut self, window: &ServoShellWindow) -> bool {
        // Note: We must use the "bitwise OR" (|) operator here instead of "logical OR" (||)
        //       because logical OR would short-circuit if any of the functions return true.
        //       We want to ensure that all functions are called. The "bitwise OR" operator
        //       does not short-circuit.
        self.update_load_status(window)
            | self.update_location_in_toolbar(window)
            | self.update_status_text(window)
            | self.update_can_go_back_and_forward(window)
    }

    /// Returns true if a redraw is required after handling the provided event.
    pub(crate) fn handle_accesskit_event(
        &mut self,
        event: &egui_winit::accesskit_winit::WindowEvent,
    ) -> bool {
        match event {
            egui_winit::accesskit_winit::WindowEvent::InitialTreeRequested => {
                self.context.egui_ctx.enable_accesskit();
                true
            },
            egui_winit::accesskit_winit::WindowEvent::ActionRequested(req) => {
                self.context
                    .egui_winit
                    .on_accesskit_action_request(req.clone());
                true
            },
            egui_winit::accesskit_winit::WindowEvent::AccessibilityDeactivated => {
                self.context.egui_ctx.disable_accesskit();
                false
            },
        }
    }

    pub(crate) fn set_zoom_factor(&self, factor: f32) {
        self.context.egui_ctx.set_zoom_factor(factor);
    }

    pub(crate) fn notify_accessibility_tree_update(&mut self, tree_update: accesskit::TreeUpdate) {
        self.pending_accesskit_updates.push(tree_update);
    }
}

fn embedder_image_to_egui_image(image: &Image) -> egui::ColorImage {
    let width = image.width as usize;
    let height = image.height as usize;

    match image.format {
        PixelFormat::K8 => egui::ColorImage::from_gray([width, height], image.data()),
        PixelFormat::KA8 => {
            // Convert to rgba
            let data: Vec<u8> = image
                .data()
                .chunks_exact(2)
                .flat_map(|pixel| [pixel[0], pixel[0], pixel[0], pixel[1]])
                .collect();
            egui::ColorImage::from_rgba_unmultiplied([width, height], &data)
        },
        PixelFormat::RGB8 => egui::ColorImage::from_rgb([width, height], image.data()),
        PixelFormat::RGBA8 => {
            egui::ColorImage::from_rgba_unmultiplied([width, height], image.data())
        },
        PixelFormat::BGRA8 => {
            // Convert from BGRA to RGBA
            let data: Vec<u8> = image
                .data()
                .chunks_exact(4)
                .flat_map(|chunk| [chunk[2], chunk[1], chunk[0], chunk[3]])
                .collect();
            egui::ColorImage::from_rgba_unmultiplied([width, height], &data)
        },
    }
}

/// Uploads all favicons that have not yet been processed to the GPU.
fn load_pending_favicons(
    ctx: &egui::Context,
    window: &ServoShellWindow,
    texture_cache: &mut HashMap<WebViewId, (egui::TextureHandle, egui::load::SizedTexture)>,
) {
    for id in window.take_pending_favicon_loads() {
        let Some(webview) = window.webview_by_id(id) else {
            continue;
        };
        let Some(favicon) = webview.favicon() else {
            continue;
        };

        let egui_image = embedder_image_to_egui_image(&favicon);
        let handle = ctx.load_texture(format!("favicon-{id:?}"), egui_image, Default::default());
        let texture = egui::load::SizedTexture::new(
            handle.id(),
            egui::vec2(favicon.width as f32, favicon.height as f32),
        );

        // We don't need the handle anymore but we can't drop it either since that would cause
        // the texture to be freed.
        texture_cache.insert(id, (handle, texture));
    }
}
