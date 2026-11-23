/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::HashSet;
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
use std::path::PathBuf;

#[cfg(any(
    target_os = "windows",
    target_os = "linux",
    target_os = "freebsd",
    target_os = "macos"
))]
use log::warn;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::desktop::theme::ThemePreset;

pub(crate) const LIBRENAV_PROFILE_DIRECTORY: &str = "librenav";
pub(crate) const LIBRENAV_BOOKMARKS_FILE: &str = "bookmarks.json";
pub(crate) const LIBRENAV_HISTORY_FILE: &str = "history.json";
pub(crate) const LIBRENAV_SETTINGS_FILE: &str = "browser-settings.json";
pub(crate) const LIBRENAV_EXTENSIONS_DIRECTORY: &str = "extensions";
pub(crate) const LIBRENAV_EXTENSIONS_STATE_FILE: &str = "extensions-state.json";

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
    pub(crate) theme_preset: ThemePreset,
    pub(crate) active_theme_extension: Option<String>,
    pub(crate) show_home_button: bool,
    pub(crate) show_bookmarks_bar: bool,
    pub(crate) homepage: String,
    pub(crate) searchpage: String,
    pub(crate) ublock_enabled: bool,
    pub(crate) ublock_disabled_hosts: HashSet<String>,
    pub(crate) content_scripts_enabled: bool,
}

impl Default for BrowserSettings {
    fn default() -> Self {
        Self {
            language: "fr".into(),
            theme_preset: ThemePreset::Dark,
            active_theme_extension: None,
            show_home_button: true,
            show_bookmarks_bar: true,
            homepage: "servo:newtab".into(),
            searchpage: "https://revsearch.revlibertaire.com/search.php?q=%s".into(),
            ublock_enabled: true,
            ublock_disabled_hosts: HashSet::new(),
            content_scripts_enabled: true,
        }
    }
}

impl BrowserSettings {
    pub(crate) fn homepage_is_valid(&self) -> bool {
        Url::parse(&self.homepage).is_ok()
    }

    pub(crate) fn searchpage_is_valid(&self) -> bool {
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
    ThemeLabel,
    ThemeDark,
    ThemeLight,
    ThemeHighContrast,
    ThemeNord,
    ApplyTheme,
    ResetTheme,
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
    InactiveStatus,
    Enable,
    Disable,
    ContentScriptsLabel,
    ContentScriptsDesc,
    BlockedRequestsLabel,
    BookmarksBarEmpty,
    InvalidHomepage,
    InvalidSearchpage,
    WebExtensionBadge,
    UserscriptBadge,
    ThemeBadge,
    ManageExtensions,
    NoExtensions,
    UblockPowerOn,
    UblockPowerOff,
    InstallExtension,
    InstallFromChromeStore,
    InstallFromFirefoxAddons,
    InstallLocalFile,
    ExtensionInputPlaceholder,
    PopularExtensions,
    InstalledBadge,
    UblockOpenPopup,
}

pub(crate) fn t(text: GuiText, lang: impl AsRef<str>) -> &'static str {
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
            GuiText::Appearance => "Apparence et thème",
            GuiText::ThemeLabel => "Thème de l'interface",
            GuiText::ThemeDark => "Sombre",
            GuiText::ThemeLight => "Clair",
            GuiText::ThemeHighContrast => "Contraste élevé",
            GuiText::ThemeNord => "Nord",
            GuiText::ApplyTheme => "Appliquer ce thème",
            GuiText::ResetTheme => "Revenir au thème prédéfini",
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
            GuiText::ExtensionsSubtitle => "Bloqueur intégré, thèmes d'apparence, scripts de contenu et userscripts",
            GuiText::OpenExtensionsDir => "Ouvrir le dossier des extensions",
            GuiText::OpenChromeStore => "Chrome Web Store",
            GuiText::OpenFirefoxAddons => "Firefox Add-ons",
            GuiText::OpenGreasyFork => "GreasyFork (scripts)",
            GuiText::InstalledExtensionsCount => "Extensions installées",
            GuiText::ActiveStatus => "Actif",
            GuiText::InactiveStatus => "Désactivé",
            GuiText::Enable => "Activer",
            GuiText::Disable => "Désactiver",
            GuiText::ContentScriptsLabel => "Exécuter les scripts de contenu et styles d'extensions",
            GuiText::ContentScriptsDesc => "Injecte les fichiers content_scripts (JS/CSS) et .user.js sur les pages correspondantes",
            GuiText::BlockedRequestsLabel => "Requêtes bloquées",
            GuiText::BookmarksBarEmpty => "Ajoutez des favoris pour les afficher ici",
            GuiText::InvalidHomepage => "Veuillez saisir une URL valide.",
            GuiText::InvalidSearchpage => "L'adresse doit être valide et contenir %s.",
            GuiText::WebExtensionBadge => "WebExtension",
            GuiText::UserscriptBadge => "Userscript",
            GuiText::ThemeBadge => "Thème",
            GuiText::ManageExtensions => "Gérer les extensions",
            GuiText::NoExtensions => "Aucune extension installée.",
            GuiText::UblockPowerOn => "Protection active sur ce site",
            GuiText::UblockPowerOff => "Protection désactivée sur ce site",
            GuiText::InstallExtension => "Installer une extension",
            GuiText::InstallFromChromeStore => "Chrome CRX",
            GuiText::InstallFromFirefoxAddons => "Firefox XPI",
            GuiText::InstallLocalFile => "Fichier local",
            GuiText::ExtensionInputPlaceholder => "Coller une URL Chrome Web Store, Firefox Add-ons, ou chemin local (.crx, .xpi, .zip, .user.js)...",
            GuiText::PopularExtensions => "Extensions populaires en 1 clic",
            GuiText::InstalledBadge => "Installé",
            GuiText::UblockOpenPopup => "Popup uBlock Origin",
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
            GuiText::Appearance => "Appearance & Theme",
            GuiText::ThemeLabel => "Interface theme",
            GuiText::ThemeDark => "Dark",
            GuiText::ThemeLight => "Light",
            GuiText::ThemeHighContrast => "High contrast",
            GuiText::ThemeNord => "Nord",
            GuiText::ApplyTheme => "Apply theme",
            GuiText::ResetTheme => "Reset to preset theme",
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
            GuiText::ExtensionsSubtitle => "Built-in blocker, appearance themes, content scripts and userscripts",
            GuiText::OpenExtensionsDir => "Open extensions folder",
            GuiText::OpenChromeStore => "Chrome Web Store",
            GuiText::OpenFirefoxAddons => "Firefox Add-ons",
            GuiText::OpenGreasyFork => "GreasyFork (scripts)",
            GuiText::InstalledExtensionsCount => "Installed extensions",
            GuiText::ActiveStatus => "Active",
            GuiText::InactiveStatus => "Disabled",
            GuiText::Enable => "Enable",
            GuiText::Disable => "Disable",
            GuiText::ContentScriptsLabel => "Run extension content scripts and styles",
            GuiText::ContentScriptsDesc => "Injects matching content_scripts (JS/CSS) and .user.js into web pages",
            GuiText::BlockedRequestsLabel => "Blocked requests",
            GuiText::BookmarksBarEmpty => "Save bookmarks to display them here",
            GuiText::InvalidHomepage => "Please enter a valid URL.",
            GuiText::InvalidSearchpage => "Address must be valid and contain %s.",
            GuiText::WebExtensionBadge => "WebExtension",
            GuiText::UserscriptBadge => "Userscript",
            GuiText::ThemeBadge => "Theme",
            GuiText::ManageExtensions => "Manage extensions",
            GuiText::NoExtensions => "No extensions installed.",
            GuiText::UblockPowerOn => "Protection enabled on this site",
            GuiText::UblockPowerOff => "Protection disabled on this site",
            GuiText::InstallExtension => "Install an extension",
            GuiText::InstallFromChromeStore => "Chrome CRX",
            GuiText::InstallFromFirefoxAddons => "Firefox XPI",
            GuiText::InstallLocalFile => "Local file",
            GuiText::ExtensionInputPlaceholder => "Paste Chrome Web Store URL, Firefox Add-ons URL, or local path (.crx, .xpi, .zip, .user.js)...",
            GuiText::PopularExtensions => "Popular 1-click extensions",
            GuiText::InstalledBadge => "Installed",
            GuiText::UblockOpenPopup => "uBlock Origin popup",
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
            GuiText::Appearance => "Apariencia y tema",
            GuiText::ThemeLabel => "Tema de la interfaz",
            GuiText::ThemeDark => "Oscuro",
            GuiText::ThemeLight => "Claro",
            GuiText::ThemeHighContrast => "Alto contraste",
            GuiText::ThemeNord => "Nord",
            GuiText::ApplyTheme => "Aplicar tema",
            GuiText::ResetTheme => "Volver al tema predefinido",
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
            GuiText::ExtensionsSubtitle => "Bloqueador integrado, temas visuales, content scripts y userscripts",
            GuiText::OpenExtensionsDir => "Abrir carpeta de extensiones",
            GuiText::OpenChromeStore => "Chrome Web Store",
            GuiText::OpenFirefoxAddons => "Firefox Add-ons",
            GuiText::OpenGreasyFork => "GreasyFork (scripts)",
            GuiText::InstalledExtensionsCount => "Extensiones instaladas",
            GuiText::ActiveStatus => "Activo",
            GuiText::InactiveStatus => "Desactivado",
            GuiText::Enable => "Activar",
            GuiText::Disable => "Desactivar",
            GuiText::ContentScriptsLabel => "Ejecutar content scripts y estilos de extensiones",
            GuiText::ContentScriptsDesc => "Inyecta content_scripts (JS/CSS) y .user.js en páginas coincidentes",
            GuiText::BlockedRequestsLabel => "Peticiones bloqueadas",
            GuiText::BookmarksBarEmpty => "Añade marcadores para mostrarlos aquí",
            GuiText::InvalidHomepage => "Introduce una URL válida.",
            GuiText::InvalidSearchpage => "La dirección debe ser válida y contener %s.",
            GuiText::WebExtensionBadge => "WebExtension",
            GuiText::UserscriptBadge => "Userscript",
            GuiText::ThemeBadge => "Tema",
            GuiText::ManageExtensions => "Gestionar extensiones",
            GuiText::NoExtensions => "No hay extensiones instaladas.",
            GuiText::UblockPowerOn => "Protección activada en este sitio",
            GuiText::UblockPowerOff => "Protección desactivada en este sitio",
            GuiText::InstallExtension => "Instalar una extensión",
            GuiText::InstallFromChromeStore => "Chrome CRX",
            GuiText::InstallFromFirefoxAddons => "Firefox XPI",
            GuiText::InstallLocalFile => "Archivo local",
            GuiText::ExtensionInputPlaceholder => "Pegar URL de Chrome Web Store, Firefox Add-ons, o ruta local (.crx, .xpi, .zip, .user.js)...",
            GuiText::PopularExtensions => "Extensiones populares en 1 clic",
            GuiText::InstalledBadge => "Instalado",
            GuiText::UblockOpenPopup => "Popup uBlock Origin",
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
            GuiText::Appearance => "Aspekto kaj etoso",
            GuiText::ThemeLabel => "Etoso de la interfaco",
            GuiText::ThemeDark => "Malhela",
            GuiText::ThemeLight => "Hela",
            GuiText::ThemeHighContrast => "Alta kontrasto",
            GuiText::ThemeNord => "Nord",
            GuiText::ApplyTheme => "Apliki etoson",
            GuiText::ResetTheme => "Reveni al antaŭdifinita etoso",
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
            GuiText::ExtensionsTitle => "Etendaĵoj",
            GuiText::ExtensionsSubtitle => "Enkonstruita blokilo, etosoj, enhavskriptoj kaj uzantskriptoj",
            GuiText::OpenExtensionsDir => "Malfermi dosierujon de etendaĵoj",
            GuiText::OpenChromeStore => "Chrome Web Store",
            GuiText::OpenFirefoxAddons => "Firefox Add-ons",
            GuiText::OpenGreasyFork => "GreasyFork",
            GuiText::InstalledExtensionsCount => "Instalitaj etendaĵoj",
            GuiText::ActiveStatus => "Aktiva",
            GuiText::InactiveStatus => "Malaktiva",
            GuiText::Enable => "Aktivigi",
            GuiText::Disable => "Malaktivigi",
            GuiText::ContentScriptsLabel => "Ruli enhavskriptojn kaj stilojn de etendaĵoj",
            GuiText::ContentScriptsDesc => "Injektas kongruajn content_scripts (JS/CSS) kaj .user.js",
            GuiText::BlockedRequestsLabel => "Blokitaj petoj",
            GuiText::BookmarksBarEmpty => "Konservu paĝojn por montri ilin en ĉi tiu legosigna strio",
            GuiText::InvalidHomepage => "Enigu validan absolutan adreson.",
            GuiText::InvalidSearchpage => "La adreso devas esti valida kaj enhavi %s.",
            GuiText::WebExtensionBadge => "WebExtension",
            GuiText::UserscriptBadge => "Userscript",
            GuiText::ThemeBadge => "Etoso",
            GuiText::ManageExtensions => "Administri etendaĵojn",
            GuiText::NoExtensions => "Neniu etendaĵo instalita.",
            GuiText::UblockPowerOn => "Protekto aktiva en ĉi tiu retejo",
            GuiText::UblockPowerOff => "Protekto malaktiva en ĉi tiu retejo",
            GuiText::InstallExtension => "Instali etendaĵon",
            GuiText::InstallFromChromeStore => "Chrome CRX",
            GuiText::InstallFromFirefoxAddons => "Firefox XPI",
            GuiText::InstallLocalFile => "Loka dosiero",
            GuiText::ExtensionInputPlaceholder => "Algluu retadreson de Chrome Web Store, Firefox Add-ons, aŭ dosiervojon...",
            GuiText::PopularExtensions => "Popularaj etendaĵoj per 1 klako",
            GuiText::InstalledBadge => "Instalita",
            GuiText::UblockOpenPopup => "Panelo uBlock Origin",
        },
    }
}

#[cfg(any(
    target_os = "windows",
    target_os = "linux",
    target_os = "freebsd",
    target_os = "macos"
))]
pub(crate) fn profile_file_path(file_name: &str) -> Option<PathBuf> {
    dirs::data_dir().map(|directory| directory.join(LIBRENAV_PROFILE_DIRECTORY).join(file_name))
}

#[cfg(not(any(
    target_os = "windows",
    target_os = "linux",
    target_os = "freebsd",
    target_os = "macos"
)))]
pub(crate) fn profile_file_path(_file_name: &str) -> Option<PathBuf> {
    None
}

#[cfg(any(
    target_os = "windows",
    target_os = "linux",
    target_os = "freebsd",
    target_os = "macos"
))]
pub(crate) fn extensions_directory_path() -> Option<PathBuf> {
    profile_file_path(LIBRENAV_EXTENSIONS_DIRECTORY)
}

#[cfg(not(any(
    target_os = "windows",
    target_os = "linux",
    target_os = "freebsd",
    target_os = "macos"
)))]
pub(crate) fn extensions_directory_path() -> Option<std::path::PathBuf> {
    None
}

#[cfg(any(
    target_os = "windows",
    target_os = "linux",
    target_os = "freebsd",
    target_os = "macos"
))]
pub(crate) fn load_bookmarks() -> HashSet<String> {
    let Some(path) = profile_file_path(LIBRENAV_BOOKMARKS_FILE) else {
        return HashSet::new();
    };
    let Ok(contents) = fs::read_to_string(&path) else {
        return HashSet::new();
    };
    match serde_json::from_str::<Vec<String>>(&contents) {
        Ok(urls) => urls.into_iter().collect(),
        Err(error) => {
            warn!("Could not parse bookmarks file {}: {error}", path.display());
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
pub(crate) fn load_bookmarks() -> HashSet<String> {
    HashSet::new()
}

#[cfg(any(
    target_os = "windows",
    target_os = "linux",
    target_os = "freebsd",
    target_os = "macos"
))]
pub(crate) fn save_bookmarks(bookmarks: &HashSet<String>) {
    let Some(path) = profile_file_path(LIBRENAV_BOOKMARKS_FILE) else {
        return;
    };
    if let Some(parent) = path.parent()
        && let Err(error) = fs::create_dir_all(parent)
    {
        warn!(
            "Could not create profile directory {}: {error}",
            parent.display()
        );
        return;
    }
    let mut urls: Vec<&String> = bookmarks.iter().collect();
    urls.sort_unstable();
    let Ok(json) = serde_json::to_string_pretty(&urls) else {
        return;
    };
    if let Err(error) = fs::write(&path, json) {
        warn!("Could not write bookmarks file {}: {error}", path.display());
    }
}

#[cfg(not(any(
    target_os = "windows",
    target_os = "linux",
    target_os = "freebsd",
    target_os = "macos"
)))]
pub(crate) fn save_bookmarks(_bookmarks: &HashSet<String>) {}

#[cfg(any(
    target_os = "windows",
    target_os = "linux",
    target_os = "freebsd",
    target_os = "macos"
))]
pub(crate) fn load_json_profile<T: serde::de::DeserializeOwned + Default>(file_name: &str) -> T {
    let Some(path) = profile_file_path(file_name) else {
        return T::default();
    };
    let Ok(contents) = fs::read_to_string(&path) else {
        return T::default();
    };
    match serde_json::from_str::<T>(&contents) {
        Ok(value) => value,
        Err(error) => {
            warn!("Could not parse profile file {}: {error}", path.display());
            T::default()
        },
    }
}

#[cfg(not(any(
    target_os = "windows",
    target_os = "linux",
    target_os = "freebsd",
    target_os = "macos"
)))]
pub(crate) fn load_json_profile<T: Default>(_file_name: &str) -> T {
    T::default()
}

#[cfg(any(
    target_os = "windows",
    target_os = "linux",
    target_os = "freebsd",
    target_os = "macos"
))]
pub(crate) fn save_json_profile<T: Serialize>(file_name: &str, value: &T) {
    let Some(path) = profile_file_path(file_name) else {
        return;
    };
    if let Some(parent) = path.parent()
        && let Err(error) = fs::create_dir_all(parent)
    {
        warn!(
            "Could not create profile directory {}: {error}",
            parent.display()
        );
        return;
    }
    let Ok(json) = serde_json::to_string_pretty(value) else {
        return;
    };
    if let Err(error) = fs::write(&path, json) {
        warn!("Could not write profile file {}: {error}", path.display());
    }
}

#[cfg(not(any(
    target_os = "windows",
    target_os = "linux",
    target_os = "freebsd",
    target_os = "macos"
)))]
pub(crate) fn save_json_profile<T>(_file_name: &str, _value: &T) {}
