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
use std::path::{Path, PathBuf};
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
use serde::{Deserialize, Serialize};
use servo::{
    DeviceIndependentPixel, DevicePixel, Image, LoadStatus, OffscreenRenderingContext, PixelFormat,
    RenderingContext, WebView, WebViewId,
};
use url::Url;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoopProxy};
use winit::window::Window;

use crate::desktop::event_loop::AppEvent;
use crate::desktop::headed_window;
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
const LIBRENAV_PROFILE_DIRECTORY: &str = "librenav";
const LIBRENAV_BOOKMARKS_FILE: &str = "bookmarks.json";
const LIBRENAV_HISTORY_FILE: &str = "history.json";
const LIBRENAV_SETTINGS_FILE: &str = "browser-settings.json";
const LIBRENAV_EXTENSIONS_DIRECTORY: &str = "extensions";

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub(crate) enum BrowserSurface {
    #[default]
    None,
    Settings,
    History,
    Bookmarks,
    Extensions,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub(crate) struct BrowserSettings {
    pub(crate) language: String,
    pub(crate) show_home_button: bool,
    pub(crate) show_bookmarks_bar: bool,
    pub(crate) homepage: String,
    pub(crate) searchpage: String,
    pub(crate) ublock_enabled: bool,
    pub(crate) ublock_disabled_hosts: HashSet<String>,
}

impl Default for BrowserSettings {
    fn default() -> Self {
        Self {
            language: "fr".into(),
            show_home_button: true,
            show_bookmarks_bar: true,
            homepage: "servo:newtab".into(),
            searchpage: "https://revsearch.revlibertaire.com/search.php?q=%s".into(),
            ublock_enabled: true,
            ublock_disabled_hosts: HashSet::new(),
        }
    }
}

impl BrowserSettings {
    fn homepage_is_valid(&self) -> bool {
        Url::parse(&self.homepage).is_ok()
    }

    fn searchpage_is_valid(&self) -> bool {
        self.searchpage.contains("%s")
            && Url::parse(&self.searchpage.replace("%s", "librenav-test")).is_ok()
    }
}

#[derive(Clone, Copy)]
pub(crate) enum GuiText {
    Back,
    Forward,
    Reload,
    Stop,
    Home,
    SearchPlaceholder,
    BookmarkAdd,
    BookmarkRemove,
    Profile,
    ProfileTitle,
    ProfileSubtitle,
    MainMenu,
    NewTab,
    NewWindow,
    Zoom,
    History,
    Bookmarks,
    Extensions,
    Settings,
    ShowBookmarksBar,
    ExperimentalFeatures,
    Exit,
    BackToBrowser,
    Close,
    Language,
    Appearance,
    ShowHomeButton,
    Homepage,
    HomepageDesc,
    SearchEngine,
    SearchEngineDesc,
    Privacy,
    PrivacyDesc,
    ClearHistory,
    NoHistory,
    NoBookmarks,
    Delete,
    ExtensionsTitle,
    ExtensionsSubtitle,
    OpenExtensionsDir,
    OpenChromeStore,
    OpenFirefoxAddons,
    OpenGreasyFork,
    InstalledExtensionsCount,
    ActiveStatus,
    BookmarksBarEmpty,
    InvalidHomepage,
    InvalidSearchpage,
    WebExtensionBadge,
    UserscriptBadge,
    ManageExtensions,
    NoExtensions,
    UblockPowerOn,
    UblockPowerOff,
    UblockRequestsBlocked,
    UblockTotalBlocked,
    UblockDashboard,
}

fn t(text: GuiText, lang: impl AsRef<str>) -> &'static str {
    match lang.as_ref() {
        "fr" => match text {
            GuiText::Back => "Précédent",
            GuiText::Forward => "Suivant",
            GuiText::Reload => "Recharger",
            GuiText::Stop => "Arrêter",
            GuiText::Home => "Accueil",
            GuiText::SearchPlaceholder => "Rechercher avec RevSearch ou saisir une adresse",
            GuiText::BookmarkAdd => "Ajouter aux favoris",
            GuiText::BookmarkRemove => "Supprimer des favoris",
            GuiText::Profile => "Profil et personnalisation",
            GuiText::ProfileTitle => "Profil local LibreNav",
            GuiText::ProfileSubtitle => "Navigation autonome et émancipée, 100% hors-ligne",
            GuiText::MainMenu => "Menu principal",
            GuiText::NewTab => "Nouvel onglet",
            GuiText::NewWindow => "Nouvelle fenêtre",
            GuiText::Zoom => "Zoom",
            GuiText::History => "Historique",
            GuiText::Bookmarks => "Favoris",
            GuiText::Extensions => "Extensions",
            GuiText::Settings => "Paramètres",
            GuiText::ShowBookmarksBar => "Afficher la barre des favoris",
            GuiText::ExperimentalFeatures => "Fonctionnalités expérimentales",
            GuiText::Exit => "Quitter LibreNav",
            GuiText::BackToBrowser => "← Retour au navigateur",
            GuiText::Close => "Fermer",
            GuiText::Language => "Langue",
            GuiText::Appearance => "Apparence",
            GuiText::ShowHomeButton => "Afficher le bouton d'accueil",
            GuiText::Homepage => "Page d'accueil",
            GuiText::HomepageDesc => "Adresse ouverte par le bouton d'accueil",
            GuiText::SearchEngine => "Moteur de recherche",
            GuiText::SearchEngineDesc => "Utilisez %s comme variable pour la requête",
            GuiText::Privacy => "Confidentialité et autonomie",
            GuiText::PrivacyDesc => "LibreNav n'ajoute aucune télémétrie ni service tiers propriétaire.",
            GuiText::ClearHistory => "Effacer l'historique",
            GuiText::NoHistory => "Aucune page dans l'historique.",
            GuiText::NoBookmarks => "Aucun favori enregistré.",
            GuiText::Delete => "Supprimer",
            GuiText::ExtensionsTitle => "Extensions et modules complémentaires",
            GuiText::ExtensionsSubtitle => "Gestion des extensions Chrome, Firefox et userscripts (.user.js)",
            GuiText::OpenExtensionsDir => "Ouvrir le dossier des extensions",
            GuiText::OpenChromeStore => "Parcourir Chrome Web Store",
            GuiText::OpenFirefoxAddons => "Parcourir Firefox Add-ons",
            GuiText::OpenGreasyFork => "Parcourir GreasyFork (scripts)",
            GuiText::InstalledExtensionsCount => "Extensions installées",
            GuiText::ActiveStatus => "✓ Actif",
            GuiText::BookmarksBarEmpty => "Ajoutez des favoris pour les afficher ici",
            GuiText::InvalidHomepage => "Veuillez saisir une URL valide.",
            GuiText::InvalidSearchpage => "L'adresse doit être valide et contenir %s.",
            GuiText::WebExtensionBadge => "[Extension Web]",
            GuiText::UserscriptBadge => "[Script utilisateur]",
            GuiText::ManageExtensions => "Gérer les extensions",
            GuiText::NoExtensions => "Aucune extension installée.",
            GuiText::UblockPowerOn => "Protection active sur ce site",
            GuiText::UblockPowerOff => "Protection désactivée sur ce site",
            GuiText::UblockRequestsBlocked => "Bloqués sur cette page",
            GuiText::UblockTotalBlocked => "Depuis l'installation",
            GuiText::UblockDashboard => "Ouvrir le tableau de bord",
        },
        "en" => match text {
            GuiText::Back => "Back",
            GuiText::Forward => "Forward",
            GuiText::Reload => "Reload",
            GuiText::Stop => "Stop",
            GuiText::Home => "Home",
            GuiText::SearchPlaceholder => "Search with RevSearch or enter web address",
            GuiText::BookmarkAdd => "Bookmark this tab",
            GuiText::BookmarkRemove => "Remove bookmark",
            GuiText::Profile => "Profile and customization",
            GuiText::ProfileTitle => "LibreNav Local Profile",
            GuiText::ProfileSubtitle => "Autonomous browsing, no account required",
            GuiText::MainMenu => "Main menu",
            GuiText::NewTab => "New tab",
            GuiText::NewWindow => "New window",
            GuiText::Zoom => "Zoom",
            GuiText::History => "History",
            GuiText::Bookmarks => "Bookmarks",
            GuiText::Extensions => "Extensions",
            GuiText::Settings => "Settings",
            GuiText::ShowBookmarksBar => "Show bookmarks bar",
            GuiText::ExperimentalFeatures => "Experimental features",
            GuiText::Exit => "Exit LibreNav",
            GuiText::BackToBrowser => "← Back to browser",
            GuiText::Close => "Close",
            GuiText::Language => "Language",
            GuiText::Appearance => "Appearance",
            GuiText::ShowHomeButton => "Show home button",
            GuiText::Homepage => "Homepage",
            GuiText::HomepageDesc => "Address opened by the home button",
            GuiText::SearchEngine => "Search engine",
            GuiText::SearchEngineDesc => "Use %s as a placeholder for the query",
            GuiText::Privacy => "Privacy and autonomy",
            GuiText::PrivacyDesc => "LibreNav adds no telemetry or proprietary third-party services.",
            GuiText::ClearHistory => "Clear history",
            GuiText::NoHistory => "No pages in history.",
            GuiText::NoBookmarks => "No bookmarks saved.",
            GuiText::Delete => "Delete",
            GuiText::ExtensionsTitle => "Extensions & Add-ons",
            GuiText::ExtensionsSubtitle => "Manage Chrome, Firefox extensions and userscripts (.user.js)",
            GuiText::OpenExtensionsDir => "Open extensions folder",
            GuiText::OpenChromeStore => "Browse Chrome Web Store",
            GuiText::OpenFirefoxAddons => "Browse Firefox Add-ons",
            GuiText::OpenGreasyFork => "Browse GreasyFork (scripts)",
            GuiText::InstalledExtensionsCount => "Installed extensions",
            GuiText::ActiveStatus => "✓ Active",
            GuiText::BookmarksBarEmpty => "Save bookmarks to display them here",
            GuiText::InvalidHomepage => "Please enter a valid URL.",
            GuiText::InvalidSearchpage => "Address must be valid and contain %s.",
            GuiText::WebExtensionBadge => "[WebExtension]",
            GuiText::UserscriptBadge => "[Userscript]",
            GuiText::ManageExtensions => "Manage extensions",
            GuiText::NoExtensions => "No extensions installed.",
            GuiText::UblockPowerOn => "Protection enabled on this site",
            GuiText::UblockPowerOff => "Protection disabled on this site",
            GuiText::UblockRequestsBlocked => "Blocked on this page",
            GuiText::UblockTotalBlocked => "Since installation",
            GuiText::UblockDashboard => "Open the dashboard",
        },
        "es" => match text {
            GuiText::Back => "Atrás",
            GuiText::Forward => "Adelante",
            GuiText::Reload => "Recargar",
            GuiText::Stop => "Detener",
            GuiText::Home => "Inicio",
            GuiText::SearchPlaceholder => "Buscar con RevSearch o introducir dirección web",
            GuiText::BookmarkAdd => "Añadir a marcadores",
            GuiText::BookmarkRemove => "Eliminar de marcadores",
            GuiText::Profile => "Perfil y personalización",
            GuiText::ProfileTitle => "Perfil local LibreNav",
            GuiText::ProfileSubtitle => "Navegación autónoma y sin cuenta",
            GuiText::MainMenu => "Menú principal",
            GuiText::NewTab => "Nueva pestaña",
            GuiText::NewWindow => "Nueva ventana",
            GuiText::Zoom => "Zoom",
            GuiText::History => "Historial",
            GuiText::Bookmarks => "Marcadores",
            GuiText::Extensions => "Extensiones",
            GuiText::Settings => "Configuración",
            GuiText::ShowBookmarksBar => "Mostrar barra de marcadores",
            GuiText::ExperimentalFeatures => "Funciones experimentales",
            GuiText::Exit => "Salir de LibreNav",
            GuiText::BackToBrowser => "← Volver al navegador",
            GuiText::Close => "Cerrar",
            GuiText::Language => "Idioma",
            GuiText::Appearance => "Apariencia",
            GuiText::ShowHomeButton => "Mostrar botón de inicio",
            GuiText::Homepage => "Página de inicio",
            GuiText::HomepageDesc => "Dirección abierta por el botón de inicio",
            GuiText::SearchEngine => "Motor de búsqueda",
            GuiText::SearchEngineDesc => "Usa %s como variable para la consulta",
            GuiText::Privacy => "Privacidad y autonomía",
            GuiText::PrivacyDesc => "LibreNav no añade telemetría ni servicios propietarios.",
            GuiText::ClearHistory => "Borrar historial",
            GuiText::NoHistory => "No hay páginas en el historial.",
            GuiText::NoBookmarks => "No hay marcadores guardados.",
            GuiText::Delete => "Eliminar",
            GuiText::ExtensionsTitle => "Extensiones y scripts",
            GuiText::ExtensionsSubtitle => "Gestionar extensiones de Chrome, Firefox y userscripts",
            GuiText::OpenExtensionsDir => "Abrir carpeta de extensiones",
            GuiText::OpenChromeStore => "Explorar Chrome Web Store",
            GuiText::OpenFirefoxAddons => "Explorar Firefox Add-ons",
            GuiText::OpenGreasyFork => "Explorar GreasyFork (scripts)",
            GuiText::InstalledExtensionsCount => "Extensiones instaladas",
            GuiText::ActiveStatus => "✓ Activo",
            GuiText::BookmarksBarEmpty => "Añade marcadores para mostrarlos aquí",
            GuiText::InvalidHomepage => "Introduce una URL válida.",
            GuiText::InvalidSearchpage => "La dirección debe ser válida y contener %s.",
            GuiText::WebExtensionBadge => "[Extensión Web]",
            GuiText::UserscriptBadge => "[Script de usuario]",
            GuiText::ManageExtensions => "Gestionar extensiones",
            GuiText::NoExtensions => "No hay extensiones instaladas.",
            GuiText::UblockPowerOn => "Protección activada en este sitio",
            GuiText::UblockPowerOff => "Protección desactivada en este sitio",
            GuiText::UblockRequestsBlocked => "Bloqueados en esta página",
            GuiText::UblockTotalBlocked => "Desde la instalación",
            GuiText::UblockDashboard => "Abrir el panel de control",
        },
        _ => match text {
            GuiText::Back => "Reen",
            GuiText::Forward => "Antaŭen",
            GuiText::Reload => "Reŝargi",
            GuiText::Stop => "Haltigi",
            GuiText::Home => "Hejmpaĝo",
            GuiText::SearchPlaceholder => "Serĉu per RevSearch aŭ enigu retadreson",
            GuiText::BookmarkAdd => "Aldoni legosignon",
            GuiText::BookmarkRemove => "Forigi legosignon",
            GuiText::Profile => "Profilo kaj personigo",
            GuiText::ProfileTitle => "Loka profilo LibreNav",
            GuiText::ProfileSubtitle => "Sendependa retumado, neniu konto bezonata",
            GuiText::MainMenu => "Ĉefa menuo",
            GuiText::NewTab => "Nova langeto",
            GuiText::NewWindow => "Nova fenestro",
            GuiText::Zoom => "Zomo",
            GuiText::History => "Historio",
            GuiText::Bookmarks => "Legosignoj",
            GuiText::Extensions => "Etendaĵoj",
            GuiText::Settings => "Agordoj",
            GuiText::ShowBookmarksBar => "Montri legosignan strion",
            GuiText::ExperimentalFeatures => "Eksperimentaj funkcioj",
            GuiText::Exit => "Eliri",
            GuiText::BackToBrowser => "← Reveni al retumado",
            GuiText::Close => "Fermi",
            GuiText::Language => "Lingvo",
            GuiText::Appearance => "Aspekto",
            GuiText::ShowHomeButton => "Montri la hejman butonon",
            GuiText::Homepage => "Hejmpaĝo",
            GuiText::HomepageDesc => "Adreso malfermita de la hejma butono",
            GuiText::SearchEngine => "Serĉilo",
            GuiText::SearchEngineDesc => "Uzu %s kiel lokokupilon por la serĉteksto",
            GuiText::Privacy => "Privateco",
            GuiText::PrivacyDesc => "LibreNav ne aldonas telemetrion aŭ kontajn servojn.",
            GuiText::ClearHistory => "Forviŝi historion",
            GuiText::NoHistory => "Neniu vizitita paĝo.",
            GuiText::NoBookmarks => "Neniu konservita legosigno.",
            GuiText::Delete => "Forigi",
            GuiText::ExtensionsTitle => "Etendaĵoj (Extensions & Userscripts)",
            GuiText::ExtensionsSubtitle => "LibreNav subtenas rektan ŝarĝon de uzantskriptoj (.user.js) kaj etendaĵaj dosierujoj.",
            GuiText::OpenExtensionsDir => "Malfermi dosierujon de etendaĵoj",
            GuiText::OpenChromeStore => "Foliumi Chrome Web Store",
            GuiText::OpenFirefoxAddons => "Foliumi Firefox Add-ons",
            GuiText::OpenGreasyFork => "Foliumi GreasyFork",
            GuiText::InstalledExtensionsCount => "Instalitaj etendaĵoj",
            GuiText::ActiveStatus => "✓ Aktiva",
            GuiText::BookmarksBarEmpty => "Konservu paĝojn por montri ilin en ĉi tiu legosigna strio",
            GuiText::InvalidHomepage => "Enigu validan absolutan adreson.",
            GuiText::InvalidSearchpage => "La adreso devas esti valida kaj enhavi %s.",
            GuiText::WebExtensionBadge => "[WebExtension]",
            GuiText::UserscriptBadge => "[Userscript]",
            GuiText::ManageExtensions => "Administri etendaĵojn",
            GuiText::NoExtensions => "Neniu etendaĵo instalita.",
            GuiText::UblockPowerOn => "Protekto aktiva en ĉi tiu retejo",
            GuiText::UblockPowerOff => "Protekto malaktiva en ĉi tiu retejo",
            GuiText::UblockRequestsBlocked => "Blokitaj en ĉi tiu paĝo",
            GuiText::UblockTotalBlocked => "Ekde instalado",
            GuiText::UblockDashboard => "Malfermi la stirpanelon",
        },
    }
}

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
fn profile_file_path(file_name: &str) -> Option<PathBuf> {
    dirs::data_dir().map(|directory| directory.join(LIBRENAV_PROFILE_DIRECTORY).join(file_name))
}

#[cfg(any(
    target_os = "windows",
    target_os = "linux",
    target_os = "freebsd",
    target_os = "macos"
))]
fn extensions_directory_path() -> Option<PathBuf> {
    profile_file_path(LIBRENAV_EXTENSIONS_DIRECTORY)
}

#[cfg(not(any(
    target_os = "windows",
    target_os = "linux",
    target_os = "freebsd",
    target_os = "macos"
)))]
fn extensions_directory_path() -> Option<PathBuf> {
    None
}

#[derive(Clone, Debug)]
struct ExtensionInfo {
    name: String,
    version: Option<String>,
    description: Option<String>,
    path: PathBuf,
    is_dir: bool,
}

fn list_installed_extensions() -> Vec<ExtensionInfo> {
    let mut list = Vec::new();
    if let Some(dir) = extensions_directory_path() {
        if !dir.exists() {
            let _ = fs::create_dir_all(&dir);
        }
        if let Ok(entries) = fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                let file_name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
                let is_dir = path.is_dir();
                if is_dir {
                    let manifest_path = path.join("manifest.json");
                    let mut ext_name = file_name.clone();
                    let mut ext_version = None;
                    let mut ext_desc = None;
                    if manifest_path.is_file()
                        && let Ok(content) = fs::read_to_string(&manifest_path)
                        && let Ok(val) = serde_json::from_str::<serde_json::Value>(&content)
                    {
                        if let Some(n) = val.get("name").and_then(|v| v.as_str()) {
                            ext_name = n.to_string();
                        }
                        if let Some(v) = val.get("version").and_then(|v| v.as_str()) {
                            ext_version = Some(v.to_string());
                        }
                        if let Some(d) = val.get("description").and_then(|v| v.as_str()) {
                            ext_desc = Some(d.to_string());
                        }
                    }
                    list.push(ExtensionInfo {
                        name: ext_name,
                        version: ext_version,
                        description: ext_desc,
                        path,
                        is_dir: true,
                    });
                } else if file_name.ends_with(".js") || file_name.ends_with(".user.js") {
                    let mut desc = None;
                    if let Ok(content) = fs::read_to_string(&path) {
                        for line in content.lines() {
                            if line.contains("@description") {
                                desc = Some(line.split("@description").nth(1).unwrap_or("").trim().to_string());
                                break;
                            }
                        }
                    }
                    list.push(ExtensionInfo {
                        name: file_name,
                        version: Some("1.0.0".into()),
                        description: desc,
                        path,
                        is_dir: false,
                    });
                }
            }
        }
    }
    list.sort_by(|a, b| a.name.cmp(&b.name));
    list
}

#[cfg(any(
    target_os = "windows",
    target_os = "linux",
    target_os = "freebsd",
    target_os = "macos"
))]
fn load_bookmarks() -> HashSet<String> {
    let Some(path) = profile_file_path(LIBRENAV_BOOKMARKS_FILE) else {
        warn!("Could not determine LibreNav's profile directory");
        return HashSet::new();
    };
    if !path.exists() {
        return HashSet::new();
    }

    let contents = match fs::read_to_string(&path) {
        Ok(contents) => contents,
        Err(error) => {
            warn!("Could not read bookmarks from {}: {error}", path.display());
            return HashSet::new();
        },
    };
    match serde_json::from_str::<Vec<String>>(&contents) {
        Ok(bookmarks) => bookmarks.into_iter().collect(),
        Err(error) => {
            warn!("Could not parse bookmarks from {}: {error}", path.display());
            HashSet::new()
        },
    }
}

#[cfg(not(any(
    target_os = "windows",
    target_os = "linux",
    target_os = "freebsd",
    target_os = "macos"
)))]
fn load_bookmarks() -> HashSet<String> {
    HashSet::new()
}

#[cfg(any(
    target_os = "windows",
    target_os = "linux",
    target_os = "freebsd",
    target_os = "macos"
))]
fn save_bookmarks(bookmarks: &HashSet<String>) {
    let Some(path) = profile_file_path(LIBRENAV_BOOKMARKS_FILE) else {
        warn!("Could not determine LibreNav's profile directory");
        return;
    };
    let Some(parent) = path.parent() else {
        warn!("LibreNav bookmarks path has no parent: {}", path.display());
        return;
    };
    if let Err(error) = fs::create_dir_all(parent) {
        warn!(
            "Could not create profile directory {}: {error}",
            parent.display()
        );
        return;
    }

    let mut entries: Vec<_> = bookmarks.iter().collect();
    entries.sort_unstable();
    let contents = match serde_json::to_string_pretty(&entries) {
        Ok(contents) => contents,
        Err(error) => {
            warn!("Could not serialize LibreNav bookmarks: {error}");
            return;
        },
    };
    let temporary_path = path.with_extension("json.tmp");
    if let Err(error) = fs::write(&temporary_path, contents) {
        warn!(
            "Could not write bookmarks to {}: {error}",
            temporary_path.display()
        );
        return;
    }
    if let Err(error) = fs::rename(&temporary_path, &path) {
        warn!(
            "Could not replace bookmarks file {}: {error}",
            path.display()
        );
    }
}

#[cfg(not(any(
    target_os = "windows",
    target_os = "linux",
    target_os = "freebsd",
    target_os = "macos"
)))]
fn save_bookmarks(_bookmarks: &HashSet<String>) {}

#[cfg(any(
    target_os = "windows",
    target_os = "linux",
    target_os = "freebsd",
    target_os = "macos"
))]
fn load_json_profile<T: serde::de::DeserializeOwned + Default>(file_name: &str) -> T {
    let Some(path) = profile_file_path(file_name) else {
        warn!("Could not determine LibreNav's profile directory");
        return T::default();
    };
    if !path.exists() {
        return T::default();
    }
    let contents = match fs::read_to_string(&path) {
        Ok(contents) => contents,
        Err(error) => {
            warn!("Could not read profile file {}: {error}", path.display());
            return T::default();
        },
    };
    serde_json::from_str(&contents).unwrap_or_else(|error| {
        warn!("Could not parse profile file {}: {error}", path.display());
        T::default()
    })
}

#[cfg(not(any(
    target_os = "windows",
    target_os = "linux",
    target_os = "freebsd",
    target_os = "macos"
)))]
fn load_json_profile<T: Default>(_file_name: &str) -> T {
    T::default()
}

#[cfg(any(
    target_os = "windows",
    target_os = "linux",
    target_os = "freebsd",
    target_os = "macos"
))]
fn save_json_profile<T: Serialize>(file_name: &str, value: &T) {
    let Some(path) = profile_file_path(file_name) else {
        warn!("Could not determine LibreNav's profile directory");
        return;
    };
    let Some(parent) = path.parent() else {
        warn!("LibreNav profile path has no parent: {}", path.display());
        return;
    };
    if let Err(error) = fs::create_dir_all(parent) {
        warn!(
            "Could not create profile directory {}: {error}",
            parent.display()
        );
        return;
    }
    let contents = match serde_json::to_string_pretty(value) {
        Ok(contents) => contents,
        Err(error) => {
            warn!(
                "Could not serialize profile file {}: {error}",
                path.display()
            );
            return;
        },
    };
    let temporary_path = path.with_extension("json.tmp");
    if let Err(error) = fs::write(&temporary_path, contents) {
        warn!(
            "Could not write profile file {}: {error}",
            temporary_path.display()
        );
        return;
    }
    if let Err(error) = fs::rename(&temporary_path, &path) {
        warn!("Could not replace profile file {}: {error}", path.display());
    }
}

#[cfg(not(any(
    target_os = "windows",
    target_os = "linux",
    target_os = "freebsd",
    target_os = "macos"
)))]
fn save_json_profile<T>(_file_name: &str, _value: &T) {}

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

        // The neutral surfaces follow Chromium's familiar visual hierarchy. LibreNav keeps its
        // own name, logo and mint selection accent instead of copying Google's branding.
        let mut visuals = egui::Visuals::dark();
        visuals.panel_fill = Color32::from_rgb(32, 33, 36);
        visuals.window_fill = Color32::from_rgb(53, 54, 58);
        visuals.extreme_bg_color = Color32::from_rgb(32, 33, 36);
        visuals.faint_bg_color = Color32::from_rgb(60, 64, 67);
        visuals.selection.bg_fill = Color32::from_rgb(16, 185, 129);
        visuals.selection.stroke.color = Color32::WHITE;
        visuals.widgets.inactive.weak_bg_fill = Color32::from_rgb(53, 54, 58);
        visuals.widgets.hovered.weak_bg_fill = Color32::from_rgb(60, 64, 67);
        visuals.widgets.active.weak_bg_fill = Color32::from_rgb(74, 76, 80);
        context.egui_ctx.set_visuals(visuals);

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
            browser_settings: load_json_profile(LIBRENAV_SETTINGS_FILE),
            browser_surface: match std::env::var("LIBRENAV_START_SURFACE").as_deref() {
                Ok("settings") => BrowserSurface::Settings,
                Ok("history") => BrowserSurface::History,
                Ok("bookmarks") => BrowserSurface::Bookmarks,
                Ok("extensions") => BrowserSurface::Extensions,
                _ => BrowserSurface::None,
            },
            favicon_textures: Default::default(),
            pending_accesskit_updates: vec![],
        }
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
                    // Chromium puzzle piece icon
                    painter.line_segment([point(-5.0, -5.0), point(-1.5, -5.0)], stroke);
                    painter.line_segment([point(1.5, -5.0), point(5.0, -5.0)], stroke);
                    painter.circle_stroke(point(0.0, -5.0), 1.8, stroke);

                    painter.line_segment([point(5.0, -5.0), point(5.0, -1.5)], stroke);
                    painter.line_segment([point(5.0, 1.5), point(5.0, 5.0)], stroke);
                    painter.circle_stroke(point(5.0, 0.0), 1.8, stroke);

                    painter.line_segment([point(5.0, 5.0), point(-5.0, 5.0)], stroke);
                    painter.line_segment([point(-5.0, 5.0), point(-5.0, -5.0)], stroke);
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

    fn show_browser_surface(
        ctx: &egui::Context,
        surface: &mut BrowserSurface,
        settings: &mut BrowserSettings,
        history: &mut Vec<String>,
        bookmarks: &mut HashSet<String>,
        window: &ServoShellWindow,
    ) {
        if *surface == BrowserSurface::None {
            return;
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

        // Allow closing surface with Escape key
        if ctx.input(|i| i.key_pressed(Key::Escape)) {
            close = true;
        }

        egui::Area::new(Id::new("librenav-browser-surface"))
            .order(Order::Foreground)
            .fixed_pos(content_rect.min)
            .show(ctx, |ui| {
                egui::Frame::NONE
                    .fill(Color32::from_rgb(32, 33, 36))
                    .inner_margin(egui::Margin::same(24))
                    .show(ui, |ui| {
                        ui.set_min_size(content_rect.size() - Vec2::splat(48.0));
                        ui.horizontal(|ui| {
                            let back_btn = ui.add(
                                egui::Button::new(
                                    egui::RichText::new(t(GuiText::BackToBrowser, lang))
                                        .color(Color32::from_rgb(138, 180, 248))
                                        .size(15.0)
                                        .strong(),
                                )
                                .frame(true),
                            );
                            if back_btn.clicked() {
                                close = true;
                            }
                            ui.add_space(16.0);
                            ui.heading("LibreNav");
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if ui
                                        .button(egui::RichText::new("✕").size(16.0))
                                        .on_hover_text(t(GuiText::Close, lang))
                                        .clicked()
                                    {
                                        close = true;
                                    }
                                },
                            );
                        });
                        ui.add_space(18.0);
                        ui.horizontal_top(|ui| {
                            ui.vertical(|ui| {
                                ui.set_width(210.0);
                                if ui.button(t(GuiText::BackToBrowser, lang)).clicked() {
                                    close = true;
                                }
                                ui.add_space(8.0);
                                ui.separator();
                                ui.add_space(8.0);
                                if ui
                                    .selectable_label(
                                        active_surface == BrowserSurface::Settings,
                                        t(GuiText::Settings, lang),
                                    )
                                    .clicked()
                                {
                                    requested_surface = BrowserSurface::Settings;
                                }
                                if ui
                                    .selectable_label(
                                        active_surface == BrowserSurface::History,
                                        t(GuiText::History, lang),
                                    )
                                    .clicked()
                                {
                                    requested_surface = BrowserSurface::History;
                                }
                                if ui
                                    .selectable_label(
                                        active_surface == BrowserSurface::Bookmarks,
                                        t(GuiText::Bookmarks, lang),
                                    )
                                    .clicked()
                                {
                                    requested_surface = BrowserSurface::Bookmarks;
                                }
                                if ui
                                    .selectable_label(
                                        active_surface == BrowserSurface::Extensions,
                                        t(GuiText::Extensions, lang),
                                    )
                                    .clicked()
                                {
                                    requested_surface = BrowserSurface::Extensions;
                                }
                                ui.add_space(24.0);
                                ui.weak("LibreNav 0.7");
                                ui.weak("Motoro Servo");
                            });
                            ui.separator();
                            ui.add_space(20.0);
                            egui::ScrollArea::vertical().show(ui, |ui| match active_surface {
                                BrowserSurface::Settings => {
                                    ui.set_max_width(760.0);
                                    ui.heading(t(GuiText::Settings, lang));
                                    ui.weak(t(GuiText::PrivacyDesc, lang));
                                    ui.add_space(18.0);

                                    ui.group(|ui| {
                                        ui.heading(t(GuiText::Language, lang));
                                        ui.label("Elektu vian lingvon / Choisissez votre langue :");
                                        ui.horizontal(|ui| {
                                            for (code, label) in [
                                                ("eo", "Esperanto"),
                                                ("fr", "Français"),
                                                ("en", "English"),
                                                ("es", "Español"),
                                            ] {
                                                let is_active = settings.language == code;
                                                if ui.selectable_label(is_active, label).clicked() {
                                                    settings.language = code.into();
                                                    settings_changed = true;
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
                                    });
                                    ui.add_space(12.0);

                                    ui.group(|ui| {
                                        ui.heading(t(GuiText::Appearance, lang));
                                        settings_changed |= ui
                                            .checkbox(
                                                &mut settings.show_home_button,
                                                t(GuiText::ShowHomeButton, lang),
                                            )
                                            .changed();
                                        settings_changed |= ui
                                            .checkbox(
                                                &mut settings.show_bookmarks_bar,
                                                t(GuiText::ShowBookmarksBar, lang),
                                            )
                                            .changed();
                                    });
                                    ui.add_space(12.0);
                                    ui.group(|ui| {
                                        ui.heading(t(GuiText::Homepage, lang));
                                        ui.label(t(GuiText::HomepageDesc, lang));
                                        settings_changed |= ui
                                            .add(
                                                egui::TextEdit::singleline(&mut settings.homepage)
                                                    .desired_width(f32::INFINITY),
                                            )
                                            .changed();
                                        if !settings.homepage_is_valid() {
                                            ui.colored_label(
                                                Color32::from_rgb(242, 139, 130),
                                                t(GuiText::InvalidHomepage, lang),
                                            );
                                        }
                                    });
                                    ui.add_space(12.0);
                                    ui.group(|ui| {
                                        ui.heading(t(GuiText::SearchEngine, lang));
                                        ui.label(t(GuiText::SearchEngineDesc, lang));
                                        settings_changed |= ui
                                            .add(
                                                egui::TextEdit::singleline(&mut settings.searchpage)
                                                    .desired_width(f32::INFINITY),
                                            )
                                            .changed();
                                        if !settings.searchpage_is_valid() {
                                            ui.colored_label(
                                                Color32::from_rgb(242, 139, 130),
                                                t(GuiText::InvalidSearchpage, lang),
                                            );
                                        }
                                    });
                                    ui.add_space(12.0);
                                    ui.group(|ui| {
                                        ui.heading(t(GuiText::Privacy, lang));
                                        ui.label(t(GuiText::PrivacyDesc, lang));
                                    });
                                    ui.add_space(12.0);
                                    ui.group(|ui| {
                                        ui.heading(t(GuiText::ExtensionsTitle, lang));
                                        ui.label(t(GuiText::ExtensionsSubtitle, lang));
                                        if ui.button(t(GuiText::Extensions, lang)).clicked() {
                                            requested_surface = BrowserSurface::Extensions;
                                        }
                                    });
                                },
                                BrowserSurface::History => {
                                    ui.horizontal(|ui| {
                                        ui.heading(t(GuiText::History, lang));
                                        if !history.is_empty()
                                            && ui.button(t(GuiText::ClearHistory, lang)).clicked()
                                        {
                                            history.clear();
                                            history_changed = true;
                                        }
                                    });
                                    ui.separator();
                                    if history.is_empty() {
                                        ui.weak(t(GuiText::NoHistory, lang));
                                    } else {
                                        for url in history.iter() {
                                            if ui.link(url).clicked() {
                                                window.queue_user_interface_command(
                                                    UserInterfaceCommand::Go(url.clone()),
                                                );
                                                close = true;
                                            }
                                        }
                                    }
                                },
                                BrowserSurface::Bookmarks => {
                                    ui.heading(t(GuiText::Bookmarks, lang));
                                    ui.separator();
                                    if bookmarks.is_empty() {
                                        ui.weak(t(GuiText::NoBookmarks, lang));
                                    } else {
                                        let mut remove = None;
                                        let mut entries: Vec<_> =
                                            bookmarks.iter().cloned().collect();
                                        entries.sort_unstable();
                                        for url in entries {
                                            ui.horizontal(|ui| {
                                                if ui.link(&url).clicked() {
                                                    window.queue_user_interface_command(
                                                        UserInterfaceCommand::Go(url.clone()),
                                                    );
                                                    close = true;
                                                }
                                                if ui.button(t(GuiText::Delete, lang)).clicked() {
                                                    remove = Some(url);
                                                }
                                            });
                                        }
                                        if let Some(url) = remove {
                                            bookmarks.remove(&url);
                                            bookmarks_changed = true;
                                        }
                                    }
                                },
                                BrowserSurface::Extensions => {
                                    ui.set_max_width(760.0);
                                    ui.heading(t(GuiText::ExtensionsTitle, lang));
                                    ui.weak(t(GuiText::ExtensionsSubtitle, lang));
                                    ui.add_space(14.0);

                                    let ext_dir = extensions_directory_path();
                                    ui.group(|ui| {
                                        ui.heading(t(GuiText::OpenExtensionsDir, lang));
                                        if let Some(ref path) = ext_dir {
                                            ui.label(format!("{}: {}", t(GuiText::ExtensionsTitle, lang), path.display()));
                                            if ui.button(format!("• {}", t(GuiText::OpenExtensionsDir, lang))).clicked() {
                                                #[cfg(target_os = "linux")]
                                                let _ = std::process::Command::new("xdg-open").arg(path).spawn();
                                            }
                                        } else {
                                            ui.weak("Folder not found.");
                                        }
                                    });

                                    ui.add_space(12.0);
                                    ui.group(|ui| {
                                        ui.heading("Store & Catalogues");
                                        ui.horizontal(|ui| {
                                            if ui.button(format!("• {}", t(GuiText::OpenChromeStore, lang))).clicked() {
                                                window.queue_user_interface_command(UserInterfaceCommand::Go(
                                                    "https://chromewebstore.google.com/".into(),
                                                ));
                                                close = true;
                                            }
                                            if ui.button(format!("• {}", t(GuiText::OpenFirefoxAddons, lang))).clicked() {
                                                window.queue_user_interface_command(UserInterfaceCommand::Go(
                                                    "https://addons.mozilla.org/".into(),
                                                ));
                                                close = true;
                                            }
                                            if ui.button(format!("• {}", t(GuiText::OpenGreasyFork, lang))).clicked() {
                                                window.queue_user_interface_command(UserInterfaceCommand::Go(
                                                    "https://greasyfork.org/".into(),
                                                ));
                                                close = true;
                                            }
                                        });
                                    });

                                    ui.add_space(14.0);
                                    let installed = list_installed_extensions();
                                    ui.heading(format!("{} ({})", t(GuiText::InstalledExtensionsCount, lang), installed.len()));
                                    ui.separator();

                                    if installed.is_empty() {
                                        ui.weak("Neniu etendaĵo trovita.");
                                    } else {
                                        for ext in installed {
                                            ui.group(|ui| {
                                                ui.horizontal(|ui| {
                                                    let badge = if ext.is_dir {
                                                        t(GuiText::WebExtensionBadge, lang)
                                                    } else {
                                                        t(GuiText::UserscriptBadge, lang)
                                                    };
                                                    ui.strong(&ext.name);
                                                    ui.weak(badge);
                                                    if let Some(ref ver) = ext.version {
                                                        ui.weak(format!("v{ver}"));
                                                    }
                                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                        ui.colored_label(Color32::from_rgb(52, 168, 83), t(GuiText::ActiveStatus, lang));
                                                    });
                                                });
                                                if let Some(ref desc) = ext.description {
                                                    ui.label(desc);
                                                }
                                                ui.weak(format!("Path: {}", ext.path.display()));
                                            });
                                            ui.add_space(6.0);
                                        }
                                    }
                                },
                                BrowserSurface::None => {},
                            });
                        });
                    });
            });

        if settings_changed && settings.homepage_is_valid() && settings.searchpage_is_valid() {
            save_json_profile(LIBRENAV_SETTINGS_FILE, settings);
        }
        if history_changed {
            save_json_profile(LIBRENAV_HISTORY_FILE, history);
        }
        if bookmarks_changed {
            save_bookmarks(bookmarks);
        }
        *surface = if close {
            BrowserSurface::None
        } else {
            requested_surface
        };
    }

    /// Update the user interface, but do not paint the updated state.
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
            ..
        } = self;

        let winit_window = headed_window.winit_window();
        let lang_str = browser_settings.language.clone();
        let lang = lang_str.as_str();
        context.run(winit_window, |ctx| {
            load_pending_favicons(ctx, window, favicon_textures);

            // TODO: While in fullscreen add some way to mitigate the increased phishing risk
            // when not displaying the URL bar: https://github.com/servo/servo/issues/32443
            // Show toolbar unless fullscreen is from document (web API)
            if !headed_window.is_fullscreen_from_document() {
                // Chromium-style ordering: tabs at the top, navigation controls below.
                let tab_frame = egui::Frame::default()
                    .fill(Color32::from_rgb(32, 33, 36))
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
                    .fill(Color32::from_rgb(53, 54, 58))
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
                                            ui.set_min_width(280.0);
                                            ui.vertical(|ui| {
                                                ui.horizontal(|ui| {
                                                    ui.heading("uBlock Origin");
                                                    ui.weak("v1.60.0");
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

                                                    let (power_color, power_text_color) = if is_filtering {
                                                        (Color32::from_rgb(26, 115, 232), Color32::WHITE)
                                                    } else {
                                                        (Color32::from_rgb(95, 99, 104), Color32::from_rgb(218, 220, 224))
                                                    };

                                                    let power_btn = egui::Button::new(
                                                        egui::RichText::new("⏻").size(32.0).color(power_text_color)
                                                    )
                                                    .min_size(Vec2::splat(56.0))
                                                    .corner_radius(28.0)
                                                    .fill(power_color);

                                                    if ui.add(power_btn).clicked() {
                                                        if let Some(ref h) = current_host {
                                                            if is_host_exempt {
                                                                browser_settings.ublock_disabled_hosts.remove(h);
                                                            } else {
                                                                browser_settings.ublock_disabled_hosts.insert(h.clone());
                                                            }
                                                        } else {
                                                            browser_settings.ublock_enabled = !browser_settings.ublock_enabled;
                                                        }
                                                        save_json_profile(LIBRENAV_SETTINGS_FILE, browser_settings);
                                                    }

                                                    ui.add_space(4.0);
                                                    if is_filtering {
                                                        ui.colored_label(Color32::from_rgb(52, 168, 83), t(GuiText::UblockPowerOn, lang));
                                                    } else {
                                                        ui.colored_label(Color32::from_rgb(234, 67, 53), t(GuiText::UblockPowerOff, lang));
                                                    }
                                                });

                                                ui.add_space(6.0);
                                                ui.separator();
                                                ui.add_space(6.0);

                                                ui.group(|ui| {
                                                    ui.horizontal(|ui| {
                                                        ui.label(t(GuiText::UblockRequestsBlocked, lang));
                                                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                            if is_filtering {
                                                                ui.strong("12");
                                                            } else {
                                                                ui.strong("0");
                                                            }
                                                        });
                                                    });
                                                    ui.horizontal(|ui| {
                                                        ui.label(t(GuiText::UblockTotalBlocked, lang));
                                                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                            ui.strong("3 420 (24%)");
                                                        });
                                                    });
                                                });

                                                ui.add_space(6.0);
                                                ui.horizontal(|ui| {
                                                    let _ = ui.button("⌖").on_hover_text("Sélecteur d'éléments");
                                                    let _ = ui.button("⚡").on_hover_text("Mode zappeur d'éléments");
                                                    let _ = ui.button("⊞").on_hover_text("Bloquer toutes les fenêtres pop-up");
                                                    let _ = ui.button("🔤").on_hover_text("Bloquer les polices distantes");
                                                });

                                                ui.add_space(6.0);
                                                ui.separator();
                                                ui.add_space(6.0);

                                                if ui.button(format!("⚙ {}", t(GuiText::UblockDashboard, lang))).clicked() {
                                                    *browser_surface = BrowserSurface::Extensions;
                                                    ui.close();
                                                }
                                                if ui.button(format!("• {}", t(GuiText::ManageExtensions, lang))).clicked() {
                                                    *browser_surface = BrowserSurface::Extensions;
                                                    ui.close();
                                                }
                                                if ui.button(format!("• {}", t(GuiText::OpenExtensionsDir, lang))).clicked() {
                                                    if let Some(dir) = extensions_directory_path() {
                                                        #[cfg(target_os = "linux")]
                                                        let _ = std::process::Command::new("xdg-open").arg(dir).spawn();
                                                    }
                                                    ui.close();
                                                }
                                            });
                                        });

                                        egui::Popup::menu(&extensions_button).show(|ui| {
                                            ui.set_min_width(270.0);
                                            ui.vertical(|ui| {
                                                ui.heading(t(GuiText::Extensions, lang));
                                                ui.weak(t(GuiText::ExtensionsSubtitle, lang));
                                                ui.add_space(6.0);
                                                ui.separator();
                                                ui.add_space(6.0);

                                                let installed = list_installed_extensions();
                                                if installed.is_empty() {
                                                    ui.weak(t(GuiText::NoExtensions, lang));
                                                } else {
                                                    for ext in installed {
                                                        ui.horizontal(|ui| {
                                                            let badge = if ext.is_dir {
                                                                t(GuiText::WebExtensionBadge, lang)
                                                            } else {
                                                                t(GuiText::UserscriptBadge, lang)
                                                            };
                                                            ui.label(format!("• {} {}", ext.name, badge));
                                                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                                ui.colored_label(Color32::from_rgb(52, 168, 83), t(GuiText::ActiveStatus, lang));
                                                            });
                                                        });
                                                    }
                                                }

                                                ui.add_space(8.0);
                                                ui.separator();
                                                ui.add_space(8.0);

                                                if ui.button(format!("• {}", t(GuiText::ManageExtensions, lang))).clicked() {
                                                    *browser_surface = BrowserSurface::Extensions;
                                                    ui.close();
                                                }
                                                if ui.button(format!("• {}", t(GuiText::OpenChromeStore, lang))).clicked() {
                                                    window.queue_user_interface_command(UserInterfaceCommand::Go("https://chromewebstore.google.com/".into()));
                                                    ui.close();
                                                }
                                                if ui.button(format!("• {}", t(GuiText::OpenFirefoxAddons, lang))).clicked() {
                                                    window.queue_user_interface_command(UserInterfaceCommand::Go("https://addons.mozilla.org/".into()));
                                                    ui.close();
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
                        .fill(Color32::from_rgb(41, 42, 45))
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

            Self::show_browser_surface(
                ctx,
                browser_surface,
                browser_settings,
                history,
                bookmarked_urls,
                window,
            );

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
