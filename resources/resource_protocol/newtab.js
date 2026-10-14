"use strict";

/**
 * LibreNav New Tab Page Engine
 * Implements complete Google Chrome New Tab features:
 * - Direct customizable shortcuts grid (add, edit, delete, persistent)
 * - Custom wallpaper photo (file upload from device, image URL, preset themes)
 * - Material You / Chrome color accents
 * - Omnibox search with clear button
 * - Complete 4-language i18n (Esperanto default, Français, English, Español)
 * - Fully offline, zero CDN, resilient safe-storage
 */

// Safe Storage wrapper that handles opaque origins or restricted environments
const safeStorage = (() => {
  let memoryFallback = {};
  const isLocalStorageAvailable = () => {
    try {
      if (typeof window.localStorage !== "undefined") {
        const testKey = "__librenav_storage_test__";
        window.localStorage.setItem(testKey, "1");
        window.localStorage.removeItem(testKey);
        return true;
      }
    } catch {
      return false;
    }
    return false;
  };

  const hasStorage = isLocalStorageAvailable();

  return {
    getItem(key) {
      if (hasStorage) {
        try {
          return window.localStorage.getItem(key);
        } catch {
          return memoryFallback[key] || null;
        }
      }
      return memoryFallback[key] || null;
    },
    setItem(key, value) {
      if (hasStorage) {
        try {
          window.localStorage.setItem(key, value);
          return;
        } catch {
          // If quota exceeded or storage blocked, fall through
        }
      }
      memoryFallback[key] = String(value);
    },
    removeItem(key) {
      if (hasStorage) {
        try {
          window.localStorage.removeItem(key);
        } catch {
          delete memoryFallback[key];
        }
      }
      delete memoryFallback[key];
    }
  };
})();

// Multilingual i18n dictionary
const i18nMessages = Object.freeze({
  eo: {
    language: "Lingvo",
    engine: "Retumilo funkciigita de la motoro Servo",
    searchLabel: "Serĉi en la reto",
    searchPlaceholder: "Serĉu per RevSearch aŭ enigu retadreson",
    searchButton: "Serĉi",
    preferences: "Agordoj",
    information: "Informoj",
    privacy: "Neniu telemetrio estas aldonita de LibreNav.",
    addShortcut: "Aldoni ligilon",
    addShortcutTitle: "Aldoni rapidan ligilon",
    editShortcutTitle: "Modifi rapidan ligilon",
    shortcutName: "Nomo",
    namePlaceholder: "Ekz: Vikipedio",
    shortcutUrl: "URL",
    delete: "Forigi",
    cancel: "Nuligi",
    done: "Konservi",
    apply: "Apliki",
    customizeLibreNav: "Agordi LibreNav",
    customizeLibreNavTitle: "Agordi ĉi tiun paĝon",
    backgroundSection: "Fona bildo kaj foto",
    uploadFromDevice: "Alŝuti foton el aparato",
    resetBackground: "Restaŭri defaŭltan fonon",
    imageUrlLabel: "Aŭ enigu adreson de fona bildo",
    urlBgPlaceholder: "https://... aŭ file://...",
    presetThemes: "Antaŭdifinitaj etosoj",
    accentColorSection: "Akcenta koloro",
    showShortcutsToggle: "Montri rapidajn ligilojn",
    defaultShortcuts: {
      servo: "Projekto Servo",
      wiki: "Vikipedio",
      osm: "OpenStreetMap",
      revsearch: "RevSearch",
      prefs: "Agordoj",
      config: "Informoj"
    }
  },
  fr: {
    language: "Langue",
    engine: "Navigateur propulsé par le moteur Servo",
    searchLabel: "Rechercher sur le Web",
    searchPlaceholder: "Rechercher avec RevSearch ou saisir une URL",
    searchButton: "Rechercher",
    preferences: "Préférences",
    information: "Informations",
    privacy: "LibreNav n’ajoute aucune télémétrie.",
    addShortcut: "Ajouter un raccourci",
    addShortcutTitle: "Ajouter un raccourci",
    editShortcutTitle: "Modifier le raccourci",
    shortcutName: "Nom",
    namePlaceholder: "Ex : Wikipédia",
    shortcutUrl: "URL",
    delete: "Supprimer",
    cancel: "Annuler",
    done: "Terminé",
    apply: "Appliquer",
    customizeLibreNav: "Personnaliser LibreNav",
    customizeLibreNavTitle: "Personnaliser cette page",
    backgroundSection: "Arrière-plan et photo",
    uploadFromDevice: "Importer depuis l’appareil",
    resetBackground: "Restaurer l’arrière-plan par défaut",
    imageUrlLabel: "Ou saisir l’URL d’une image d’arrière-plan",
    urlBgPlaceholder: "https://... ou file://...",
    presetThemes: "Thèmes prédéfinis",
    accentColorSection: "Couleur d’accentuation",
    showShortcutsToggle: "Afficher les raccourcis",
    defaultShortcuts: {
      servo: "Projet Servo",
      wiki: "Wikipédia",
      osm: "OpenStreetMap",
      revsearch: "RevSearch",
      prefs: "Préférences",
      config: "Informations"
    }
  },
  en: {
    language: "Language",
    engine: "A browser powered by the Servo engine",
    searchLabel: "Search the Web",
    searchPlaceholder: "Search with RevSearch or type a URL",
    searchButton: "Search",
    preferences: "Preferences",
    information: "Information",
    privacy: "LibreNav adds no telemetry.",
    addShortcut: "Add shortcut",
    addShortcutTitle: "Add shortcut",
    editShortcutTitle: "Edit shortcut",
    shortcutName: "Name",
    namePlaceholder: "e.g. Wikipedia",
    shortcutUrl: "URL",
    delete: "Delete",
    cancel: "Cancel",
    done: "Done",
    apply: "Apply",
    customizeLibreNav: "Customize LibreNav",
    customizeLibreNavTitle: "Customize this page",
    backgroundSection: "Background photo and image",
    uploadFromDevice: "Upload from device",
    resetBackground: "Reset to default background",
    imageUrlLabel: "Or enter background image URL",
    urlBgPlaceholder: "https://... or file://...",
    presetThemes: "Preset themes",
    accentColorSection: "Accent color",
    showShortcutsToggle: "Show shortcuts",
    defaultShortcuts: {
      servo: "Servo Project",
      wiki: "Wikipedia",
      osm: "OpenStreetMap",
      revsearch: "RevSearch",
      prefs: "Preferences",
      config: "Information"
    }
  },
  es: {
    language: "Idioma",
    engine: "Un navegador impulsado por el motor Servo",
    searchLabel: "Buscar en la Web",
    searchPlaceholder: "Buscar con RevSearch o escribir una URL",
    searchButton: "Buscar",
    preferences: "Preferencias",
    information: "Información",
    privacy: "LibreNav no añade telemetría.",
    addShortcut: "Añadir acceso directo",
    addShortcutTitle: "Añadir acceso directo",
    editShortcutTitle: "Editar acceso directo",
    shortcutName: "Nombre",
    namePlaceholder: "Ej: Wikipedia",
    shortcutUrl: "URL",
    delete: "Eliminar",
    cancel: "Cancelar",
    done: "Listo",
    apply: "Aplicar",
    customizeLibreNav: "Personalizar LibreNav",
    customizeLibreNavTitle: "Personalizar esta página",
    backgroundSection: "Fondo de pantalla y foto",
    uploadFromDevice: "Subir desde el dispositivo",
    resetBackground: "Restablecer fondo predeterminado",
    imageUrlLabel: "O introduce la URL de una imagen de fondo",
    urlBgPlaceholder: "https://... o file://...",
    presetThemes: "Temas predefinidos",
    accentColorSection: "Color de énfasis",
    showShortcutsToggle: "Mostrar accesos directos",
    defaultShortcuts: {
      servo: "Proyecto Servo",
      wiki: "Wikipedia",
      osm: "OpenStreetMap",
      revsearch: "RevSearch",
      prefs: "Preferencias",
      config: "Información"
    }
  }
});

function getInitialLanguage() {
  try {
    const params = new URLSearchParams(window.location.search || window.location.hash.replace("#", "?"));
    const pLang = params.get("lang");
    if (pLang && i18nMessages[pLang]) return pLang;
  } catch {}
  return safeStorage.getItem("librenav_language") || "eo";
}

let currentLanguage = getInitialLanguage();

function applyLanguage(lang) {
  const selected = i18nMessages[lang] ? lang : "eo";
  currentLanguage = selected;
  safeStorage.setItem("librenav_language", selected);
  try {
    document.cookie = `librenav_language=${selected};path=/;max-age=31536000`;
  } catch {}
  const dictionary = i18nMessages[selected];
  document.documentElement.lang = selected;

  const selector = document.querySelector("#language");
  if (selector) selector.value = selected;

  document.querySelectorAll("[data-i18n]").forEach((element) => {
    const key = element.dataset.i18n;
    if (dictionary[key]) {
      element.textContent = dictionary[key];
    }
  });

  document.querySelectorAll("[data-i18n-placeholder]").forEach((element) => {
    const key = element.dataset.i18nPlaceholder;
    if (dictionary[key]) {
      element.placeholder = dictionary[key];
    }
  });

  document.querySelectorAll("[data-i18n-title]").forEach((element) => {
    const key = element.dataset.i18nTitle;
    if (dictionary[key]) {
      element.title = dictionary[key];
      element.setAttribute("aria-label", dictionary[key]);
    }
  });

  renderShortcuts();
}

// ----------------------------------------------------
// Shortcuts Management
// ----------------------------------------------------

const DEFAULT_SHORTCUTS = [
  { id: "servo", name: "Servo", url: "https://servo.org" },
  { id: "wiki", name: "Wikipedia", url: "https://wikipedia.org" },
  { id: "osm", name: "OpenStreetMap", url: "https://www.openstreetmap.org" },
  { id: "revsearch", name: "RevSearch", url: "https://revsearch.revlibertaire.com/" },
  { id: "prefs", name: "Preferences", url: "servo:preferences" },
  { id: "config", name: "Config", url: "servo:config" }
];

function getStoredShortcuts() {
  const stored = safeStorage.getItem("librenav_shortcuts");
  if (stored) {
    try {
      const parsed = JSON.parse(stored);
      if (Array.isArray(parsed) && parsed.length > 0) {
        return parsed;
      }
    } catch {
      // Fallback on corrupt data
    }
  }
  return DEFAULT_SHORTCUTS;
}

function saveShortcuts(shortcuts) {
  safeStorage.setItem("librenav_shortcuts", JSON.stringify(shortcuts));
  renderShortcuts();
}

function getInitialLetter(name, url) {
  if (name && name.trim().length > 0) {
    return name.trim().charAt(0).toUpperCase();
  }
  try {
    const u = new URL(url);
    return u.hostname.replace("www.", "").charAt(0).toUpperCase();
  } catch {
    return "•";
  }
}

function renderShortcuts() {
  const container = document.querySelector("#shortcuts-grid");
  if (!container) return;

  const showShortcuts = safeStorage.getItem("librenav_show_shortcuts") !== "false";
  const section = document.querySelector("#shortcuts-section");
  if (section) {
    section.style.display = showShortcuts ? "block" : "none";
  }
  if (!showShortcuts) return;

  container.innerHTML = "";
  const shortcuts = getStoredShortcuts();
  const dict = i18nMessages[currentLanguage] || i18nMessages.eo;

  shortcuts.forEach((shortcut) => {
    const item = document.createElement("div");
    item.className = "shortcut-item";
    item.dataset.id = shortcut.id;

    const link = document.createElement("a");
    link.className = "shortcut-link";
    link.href = shortcut.url;
    link.title = `${shortcut.name} (${shortcut.url})`;

    const icon = document.createElement("div");
    icon.className = "shortcut-icon";
    icon.textContent = getInitialLetter(shortcut.name, shortcut.url);

    const title = document.createElement("span");
    title.className = "shortcut-title";
    // Check if this is a default shortcut with localized name
    if (dict.defaultShortcuts && dict.defaultShortcuts[shortcut.id]) {
      title.textContent = dict.defaultShortcuts[shortcut.id];
    } else {
      title.textContent = shortcut.name;
    }

    link.appendChild(icon);
    link.appendChild(title);
    item.appendChild(link);

    // Kebab Menu Button (3 dots)
    const menuBtn = document.createElement("button");
    menuBtn.type = "button";
    menuBtn.className = "shortcut-menu-btn";
    menuBtn.setAttribute("aria-label", dict.editShortcutTitle || "Edit");
    menuBtn.title = dict.editShortcutTitle || "Edit";
    menuBtn.innerHTML = `
      <svg viewBox="0 0 24 24" width="14" height="14" fill="currentColor" aria-hidden="true">
        <circle cx="12" cy="5" r="2"></circle>
        <circle cx="12" cy="12" r="2"></circle>
        <circle cx="12" cy="19" r="2"></circle>
      </svg>
    `;
    menuBtn.addEventListener("click", (e) => {
      e.preventDefault();
      e.stopPropagation();
      openEditShortcutModal(shortcut);
    });

    item.appendChild(menuBtn);
    container.appendChild(item);
  });

  // "Add Shortcut" Button Tile
  const addItem = document.createElement("div");
  addItem.className = "shortcut-item add-shortcut-item";

  const addBtn = document.createElement("button");
  addBtn.type = "button";
  addBtn.className = "add-shortcut-btn";
  addBtn.id = "add-shortcut-trigger";
  addBtn.setAttribute("aria-label", dict.addShortcut || "Add shortcut");

  const addIcon = document.createElement("div");
  addIcon.className = "shortcut-icon plus-icon";
  addIcon.textContent = "+";

  const addTitle = document.createElement("span");
  addTitle.className = "shortcut-title";
  addTitle.textContent = dict.addShortcut || "Add shortcut";

  addBtn.appendChild(addIcon);
  addBtn.appendChild(addTitle);
  addBtn.addEventListener("click", () => openAddShortcutModal());

  addItem.appendChild(addBtn);
  container.appendChild(addItem);
}

// ----------------------------------------------------
// Shortcut Modal Logic
// ----------------------------------------------------

const shortcutModal = document.querySelector("#shortcut-modal");
const shortcutForm = document.querySelector("#shortcut-form");
const shortcutIdInput = document.querySelector("#shortcut-id");
const shortcutNameInput = document.querySelector("#shortcut-name");
const shortcutUrlInput = document.querySelector("#shortcut-url");
const shortcutModalTitle = document.querySelector("#shortcut-modal-title");
const shortcutDeleteBtn = document.querySelector("#shortcut-delete-btn");

function openAddShortcutModal() {
  const dict = i18nMessages[currentLanguage] || i18nMessages.eo;
  shortcutModalTitle.textContent = dict.addShortcutTitle;
  shortcutIdInput.value = "";
  shortcutNameInput.value = "";
  shortcutUrlInput.value = "";
  shortcutDeleteBtn.style.display = "none";
  shortcutModal.classList.add("active");
  shortcutModal.setAttribute("aria-hidden", "false");
  shortcutNameInput.focus();
}

function openEditShortcutModal(shortcut) {
  const dict = i18nMessages[currentLanguage] || i18nMessages.eo;
  shortcutModalTitle.textContent = dict.editShortcutTitle;
  shortcutIdInput.value = shortcut.id;
  shortcutNameInput.value = shortcut.name;
  shortcutUrlInput.value = shortcut.url;
  shortcutDeleteBtn.style.display = "inline-block";
  shortcutModal.classList.add("active");
  shortcutModal.setAttribute("aria-hidden", "false");
  shortcutNameInput.focus();
}

function closeShortcutModal() {
  shortcutModal.classList.remove("active");
  shortcutModal.setAttribute("aria-hidden", "true");
}

function sanitizeUrl(rawUrl) {
  let url = rawUrl.trim();
  if (!url) return "";
  if (!/^[a-zA-Z][a-zA-Z0-9+.-]*:/.test(url)) {
    url = "https://" + url;
  }
  return url;
}

if (shortcutForm) {
  shortcutForm.addEventListener("submit", (e) => {
    e.preventDefault();
    const id = shortcutIdInput.value.trim();
    const name = shortcutNameInput.value.trim();
    const url = sanitizeUrl(shortcutUrlInput.value);

    if (!name || !url) return;

    let shortcuts = getStoredShortcuts();
    if (id) {
      // Edit existing
      shortcuts = shortcuts.map((s) => (s.id === id ? { id, name, url } : s));
    } else {
      // Add new
      const newId = "shortcut_" + Date.now();
      shortcuts.push({ id: newId, name, url });
    }

    saveShortcuts(shortcuts);
    closeShortcutModal();
  });
}

if (shortcutDeleteBtn) {
  shortcutDeleteBtn.addEventListener("click", () => {
    const id = shortcutIdInput.value.trim();
    if (!id) return;
    let shortcuts = getStoredShortcuts();
    shortcuts = shortcuts.filter((s) => s.id !== id);
    saveShortcuts(shortcuts);
    closeShortcutModal();
  });
}

const shortcutCancelBtn = document.querySelector("#shortcut-cancel-btn");
const shortcutCloseX = document.querySelector("#shortcut-close-x");
if (shortcutCancelBtn) shortcutCancelBtn.addEventListener("click", closeShortcutModal);
if (shortcutCloseX) shortcutCloseX.addEventListener("click", closeShortcutModal);

// ----------------------------------------------------
// Wallpaper and Theme Customization Logic
// ----------------------------------------------------

const PRESET_BACKGROUNDS = {
  default: "radial-gradient(circle at 50% 35%, rgba(16, 185, 129, 0.12), transparent 34rem), #1f1f23",
  aurora: "linear-gradient(135deg, #09203f 0%, #537895 100%)",
  space: "radial-gradient(ellipse at bottom, #1b2735 0%, #090a0f 100%)",
  sunset: "linear-gradient(135deg, #2b1055 0%, #7597de 100%)",
  ocean: "linear-gradient(135deg, #0f2027 0%, #203a43 50%, #2c5364 100%)"
};

function applyWallpaper(wallpaper, isCustom = false) {
  if (!wallpaper || wallpaper === "default") {
    document.body.style.backgroundImage = PRESET_BACKGROUNDS.default;
    document.body.classList.remove("has-custom-bg");
    safeStorage.removeItem("librenav_custom_wallpaper");
    safeStorage.setItem("librenav_preset_theme", "default");
    return;
  }

  if (isCustom || wallpaper.startsWith("data:") || wallpaper.startsWith("http") || wallpaper.startsWith("file:")) {
    document.body.style.backgroundImage = `url("${wallpaper}")`;
    document.body.classList.add("has-custom-bg");
    safeStorage.setItem("librenav_custom_wallpaper", wallpaper);
    safeStorage.removeItem("librenav_preset_theme");
  } else if (PRESET_BACKGROUNDS[wallpaper]) {
    document.body.style.backgroundImage = PRESET_BACKGROUNDS[wallpaper];
    document.body.classList.add("has-custom-bg");
    safeStorage.removeItem("librenav_custom_wallpaper");
    safeStorage.setItem("librenav_preset_theme", wallpaper);
  }
}

function applyAccentColor(hexColor) {
  if (!hexColor) return;
  document.documentElement.style.setProperty("--accent", hexColor);
  safeStorage.setItem("librenav_accent_color", hexColor);

  // Update active swatch
  document.querySelectorAll(".color-swatch").forEach((swatch) => {
    if (swatch.dataset.color === hexColor) {
      swatch.classList.add("active");
    } else {
      swatch.classList.remove("active");
    }
  });
}

function loadSavedTheme() {
  const customBg = safeStorage.getItem("librenav_custom_wallpaper");
  const presetTheme = safeStorage.getItem("librenav_preset_theme");
  if (customBg) {
    applyWallpaper(customBg, true);
  } else if (presetTheme) {
    applyWallpaper(presetTheme, false);
  } else {
    applyWallpaper("default");
  }

  const savedColor = safeStorage.getItem("librenav_accent_color") || "#10b981";
  applyAccentColor(savedColor);

  const showShortcutsCb = document.querySelector("#toggle-shortcuts-cb");
  if (showShortcutsCb) {
    showShortcutsCb.checked = safeStorage.getItem("librenav_show_shortcuts") !== "false";
  }
}

// ----------------------------------------------------
// Customize Modal Logic
// ----------------------------------------------------

const customizeModal = document.querySelector("#customize-modal");
const customizeBtn = document.querySelector("#customize-btn");
const customizeCloseX = document.querySelector("#customize-close-x");
const customizeDoneBtn = document.querySelector("#customize-done-btn");
const bgFileInput = document.querySelector("#bg-file-input");
const btnResetBg = document.querySelector("#btn-reset-bg");
const bgUrlInput = document.querySelector("#bg-url-input");
const btnApplyUrlBg = document.querySelector("#btn-apply-url-bg");
const toggleShortcutsCb = document.querySelector("#toggle-shortcuts-cb");

function openCustomizeModal() {
  customizeModal.classList.add("active");
  customizeModal.setAttribute("aria-hidden", "false");
}

function closeCustomizeModal() {
  customizeModal.classList.remove("active");
  customizeModal.setAttribute("aria-hidden", "true");
}

if (customizeBtn) customizeBtn.addEventListener("click", openCustomizeModal);
if (customizeCloseX) customizeCloseX.addEventListener("click", closeCustomizeModal);
if (customizeDoneBtn) customizeDoneBtn.addEventListener("click", closeCustomizeModal);

// Background File Upload
if (bgFileInput) {
  bgFileInput.addEventListener("change", (e) => {
    const file = e.target.files && e.target.files[0];
    if (!file) return;

    const reader = new FileReader();
    reader.onload = (event) => {
      const dataUrl = event.target.result;
      applyWallpaper(dataUrl, true);
    };
    reader.readAsDataURL(file);
  });
}

// Reset Background
if (btnResetBg) {
  btnResetBg.addEventListener("click", () => {
    applyWallpaper("default");
    if (bgUrlInput) bgUrlInput.value = "";
  });
}

// Apply Background from URL
if (btnApplyUrlBg && bgUrlInput) {
  btnApplyUrlBg.addEventListener("click", () => {
    const url = bgUrlInput.value.trim();
    if (url) {
      applyWallpaper(url, true);
    }
  });
}

// Preset Themes
document.querySelectorAll(".preset-btn").forEach((btn) => {
  btn.addEventListener("click", () => {
    const theme = btn.dataset.theme;
    applyWallpaper(theme, false);
    document.querySelectorAll(".preset-btn").forEach((b) => b.classList.remove("active"));
    btn.classList.add("active");
  });
});

// Accent Swatches
document.querySelectorAll(".color-swatch").forEach((swatch) => {
  swatch.addEventListener("click", () => {
    applyAccentColor(swatch.dataset.color);
  });
});

// Toggle Shortcuts
if (toggleShortcutsCb) {
  toggleShortcutsCb.addEventListener("change", (e) => {
    safeStorage.setItem("librenav_show_shortcuts", e.target.checked ? "true" : "false");
    renderShortcuts();
  });
}

// ----------------------------------------------------
// Omnibox Search Clear Button
// ----------------------------------------------------

const searchInput = document.querySelector("#search-input");
const searchClearBtn = document.querySelector("#search-clear-btn");

if (searchInput && searchClearBtn) {
  const updateClearBtn = () => {
    searchClearBtn.style.display = searchInput.value.length > 0 ? "grid" : "none";
  };
  searchInput.addEventListener("input", updateClearBtn);
  searchClearBtn.addEventListener("click", () => {
    searchInput.value = "";
    updateClearBtn();
    searchInput.focus();
  });
}

const searchForm = document.querySelector("#search-form");
if (searchForm) {
  searchForm.addEventListener("submit", (e) => {
    e.preventDefault();
    const query = searchInput ? searchInput.value.trim() : "";
    if (!query) return;

    // If query is an explicit URL or domain, go directly to it
    if (/^[a-zA-Z][a-zA-Z0-9+.-]*:/.test(query)) {
      window.location.href = query;
      return;
    }
    if (!query.includes(" ") && query.includes(".") && !query.startsWith(".")) {
      window.location.href = "https://" + query;
      return;
    }

    // Direct search execution via RevSearch
    const searchUrl = "https://revsearch.revlibertaire.com/search.php?q=" + encodeURIComponent(query);
    window.location.href = searchUrl;
  });
}

// Close modals on backdrop click or Escape key
window.addEventListener("click", (e) => {
  if (e.target === shortcutModal) closeShortcutModal();
  if (e.target === customizeModal) closeCustomizeModal();
});

window.addEventListener("keydown", (e) => {
  if (e.key === "Escape") {
    closeShortcutModal();
    closeCustomizeModal();
  }
});

// Language selector change
const langSelect = document.querySelector("#language");
if (langSelect) {
  langSelect.addEventListener("change", () => applyLanguage(langSelect.value));
}

// Initial Initialization
loadSavedTheme();
applyLanguage(currentLanguage);
