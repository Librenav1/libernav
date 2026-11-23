/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{OnceLock, RwLock};

use serde::{Deserialize, Serialize};
use url::Url;

use crate::desktop::settings::{
    BrowserSettings, LIBRENAV_EXTENSIONS_STATE_FILE, extensions_directory_path, load_json_profile,
    save_json_profile,
};
use crate::desktop::theme::{ExtensionThemeColors, parse_extension_theme};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ExtensionKind {
    Theme,
    ContentScript,
    Userscript,
    WebExtension,
}

#[derive(Clone, Debug)]
pub(crate) struct ContentScriptRule {
    pub(crate) matches: Vec<String>,
    pub(crate) exclude_matches: Vec<String>,
    pub(crate) js_files: Vec<PathBuf>,
    pub(crate) css_files: Vec<PathBuf>,
}

#[derive(Clone, Debug)]
pub(crate) struct ExtensionInfo {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) version: Option<String>,
    pub(crate) description: Option<String>,
    pub(crate) path: PathBuf,
    pub(crate) kind: ExtensionKind,
    pub(crate) enabled: bool,
    pub(crate) theme_colors: Option<ExtensionThemeColors>,
    pub(crate) content_scripts: Vec<ContentScriptRule>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
struct ExtensionStateStore {
    disabled_ids: HashSet<String>,
}

#[derive(Clone, Debug)]
struct AdBlockerState {
    enabled: bool,
    disabled_hosts: HashSet<String>,
}

impl Default for AdBlockerState {
    fn default() -> Self {
        Self {
            enabled: true,
            disabled_hosts: HashSet::new(),
        }
    }
}

static ADBLOCK_STATE: OnceLock<RwLock<AdBlockerState>> = OnceLock::new();
static ADBLOCK_BLOCKED_COUNT: AtomicU64 = AtomicU64::new(0);

fn adblock_state() -> &'static RwLock<AdBlockerState> {
    ADBLOCK_STATE.get_or_init(|| RwLock::new(AdBlockerState::default()))
}

pub(crate) fn sync_adblock_settings(settings: &BrowserSettings) {
    if let Ok(mut guard) = adblock_state().write() {
        guard.enabled = settings.ublock_enabled;
        guard.disabled_hosts = settings.ublock_disabled_hosts.clone();
    }
}

pub(crate) fn blocked_requests_count() -> u64 {
    ADBLOCK_BLOCKED_COUNT.load(Ordering::Relaxed)
}

pub(crate) fn should_block_request(request_url: &Url, page_url: Option<&Url>) -> bool {
    let scheme = request_url.scheme();
    if scheme != "http" && scheme != "https" {
        return false;
    }

    let Ok(guard) = adblock_state().read() else {
        return false;
    };
    if !guard.enabled {
        return false;
    }
    if let Some(page_host) = page_url.and_then(|u| u.host_str())
        && guard.disabled_hosts.contains(page_host)
    {
        return false;
    }

    let Some(host) = request_url.host_str() else {
        return false;
    };
    let host_lower = host.to_ascii_lowercase();

    // Never block first-party main-page host requests.
    if let Some(page_host) = page_url.and_then(|u| u.host_str())
        && host_lower == page_host.to_ascii_lowercase()
    {
        return false;
    }

    if is_known_tracker_or_ad_host(&host_lower) || is_tracker_url_pattern(request_url.as_str()) {
        ADBLOCK_BLOCKED_COUNT.fetch_add(1, Ordering::Relaxed);
        return true;
    }
    false
}

fn is_known_tracker_or_ad_host(host: &str) -> bool {
    const BLOCKED_DOMAINS: &[&str] = &[
        "doubleclick.net",
        "googlesyndication.com",
        "googleadservices.com",
        "google-analytics.com",
        "googletagmanager.com",
        "googletagservices.com",
        "adservice.google.com",
        "pagead2.googlesyndication.com",
        "connect.facebook.net",
        "pixel.facebook.com",
        "analytics.twitter.com",
        "ads-twitter.com",
        "taboola.com",
        "outbrain.com",
        "criteo.com",
        "criteo.net",
        "adnxs.com",
        "pubmatic.com",
        "rubiconproject.com",
        "openx.net",
        "casalemedia.com",
        "scorecardresearch.com",
        "quantserve.com",
        "hotjar.com",
        "mixpanel.com",
        "segment.io",
        "segment.com",
        "amplitude.com",
        "clarity.ms",
        "mc.yandex.ru",
        "amazon-adsystem.com",
        "moatads.com",
        "bidswitch.net",
        "smartadserver.com",
        "teads.tv",
        "adsrvr.org",
        "demdex.net",
        "omtrdc.net",
        "chartbeat.com",
        "newrelic.com",
        "nr-data.net",
        "bugsnag.com",
    ];

    BLOCKED_DOMAINS
        .iter()
        .any(|&domain| host == domain || host.ends_with(&format!(".{domain}")))
}

fn is_tracker_url_pattern(url: &str) -> bool {
    let lower = url.to_ascii_lowercase();
    lower.contains("/pagead/")
        || lower.contains("/ads/ga-audiences")
        || lower.contains("/gtag/js")
        || lower.contains("/fbevents.js")
        || lower.contains("/analytics.js")
        || lower.contains("prebid")
}

pub(crate) fn set_extension_enabled(id: &str, enabled: bool) {
    let mut store: ExtensionStateStore = load_json_profile(LIBRENAV_EXTENSIONS_STATE_FILE);
    if enabled {
        store.disabled_ids.remove(id);
    } else {
        store.disabled_ids.insert(id.to_string());
    }
    save_json_profile(LIBRENAV_EXTENSIONS_STATE_FILE, &store);
}

fn resolve_localized_message(ext_dir: &Path, raw: &str, lang: &str) -> String {
    let trimmed = raw.trim();
    let Some(inner) = trimmed
        .strip_prefix("__MSG_")
        .and_then(|s| s.strip_suffix("__"))
    else {
        return raw.to_string();
    };
    let key_lower = inner.to_ascii_lowercase();

    let candidates = [lang, "en", "en_US", "fr"];
    for locale in candidates {
        let msg_path = ext_dir
            .join("_locales")
            .join(locale)
            .join("messages.json");
        let Ok(content) = fs::read_to_string(&msg_path) else {
            continue;
        };
        let Ok(serde_json::Value::Object(map)) = serde_json::from_str(&content) else {
            continue;
        };
        if let Some(msg) = map
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(&key_lower))
            .and_then(|(_, v)| v.get("message"))
            .and_then(|m| m.as_str())
        {
            return msg.to_string();
        }
    }
    raw.to_string()
}

fn parse_content_scripts(ext_dir: &Path, val: &serde_json::Value) -> Vec<ContentScriptRule> {
    let Some(arr) = val.get("content_scripts").and_then(|v| v.as_array()) else {
        return Vec::new();
    };
    let mut rules = Vec::new();
    for item in arr {
        let matches: Vec<String> = item
            .get("matches")
            .and_then(|v| v.as_array())
            .map(|a| {
                a.iter()
                    .filter_map(|s| s.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();
        if matches.is_empty() {
            continue;
        }
        let exclude_matches: Vec<String> = item
            .get("exclude_matches")
            .and_then(|v| v.as_array())
            .map(|a| {
                a.iter()
                    .filter_map(|s| s.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();
        let js_files: Vec<PathBuf> = item
            .get("js")
            .and_then(|v| v.as_array())
            .map(|a| {
                a.iter()
                    .filter_map(|s| s.as_str())
                    .map(|rel| ext_dir.join(rel.trim_start_matches('/')))
                    .filter(|p| p.is_file())
                    .collect()
            })
            .unwrap_or_default();
        let css_files: Vec<PathBuf> = item
            .get("css")
            .and_then(|v| v.as_array())
            .map(|a| {
                a.iter()
                    .filter_map(|s| s.as_str())
                    .map(|rel| ext_dir.join(rel.trim_start_matches('/')))
                    .filter(|p| p.is_file())
                    .collect()
            })
            .unwrap_or_default();
        if !js_files.is_empty() || !css_files.is_empty() {
            rules.push(ContentScriptRule {
                matches,
                exclude_matches,
                js_files,
                css_files,
            });
        }
    }
    rules
}

fn parse_userscript_matches(path: &Path) -> Vec<String> {
    let Ok(content) = fs::read_to_string(path) else {
        return vec!["<all_urls>".to_string()];
    };
    let mut matches = Vec::new();
    for line in content.lines().take(60) {
        let trimmed = line.trim();
        if let Some(rest) = trimmed
            .strip_prefix("// @match")
            .or_else(|| trimmed.strip_prefix("// @include"))
        {
            let pat = rest.trim();
            if !pat.is_empty() {
                matches.push(if pat == "*" {
                    "<all_urls>".to_string()
                } else {
                    pat.to_string()
                });
            }
        }
        if trimmed == "// ==/UserScript==" {
            break;
        }
    }
    if matches.is_empty() {
        matches.push("<all_urls>".to_string());
    }
    matches
}

pub(crate) fn list_installed_extensions(lang: &str) -> Vec<ExtensionInfo> {
    let mut list = Vec::new();
    let store: ExtensionStateStore = load_json_profile(LIBRENAV_EXTENSIONS_STATE_FILE);

    let Some(dir) = extensions_directory_path() else {
        return list;
    };
    if !dir.exists() {
        let _ = fs::create_dir_all(&dir);
    }
    let Ok(entries) = fs::read_dir(&dir) else {
        return list;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let file_name = entry.file_name().to_string_lossy().to_string();
        if file_name.starts_with('.') {
            continue;
        }
        let is_dir = path.is_dir();
        let enabled = !store.disabled_ids.contains(&file_name);

        if is_dir {
            let manifest_path = path.join("manifest.json");
            let mut name = file_name.clone();
            let mut version = None;
            let mut description = None;
            let mut kind = ExtensionKind::WebExtension;
            let mut content_scripts = Vec::new();
            let theme_colors = parse_extension_theme(&manifest_path);

            if let Ok(content) = fs::read_to_string(&manifest_path)
                && let Ok(val) = serde_json::from_str::<serde_json::Value>(&content)
            {
                if let Some(n) = val.get("name").and_then(|v| v.as_str()) {
                    name = resolve_localized_message(&path, n, lang);
                }
                if let Some(v) = val.get("version").and_then(|v| v.as_str()) {
                    version = Some(v.to_string());
                }
                if let Some(d) = val.get("description").and_then(|v| v.as_str()) {
                    description = Some(resolve_localized_message(&path, d, lang));
                }
                content_scripts = parse_content_scripts(&path, &val);
                if theme_colors.is_some() {
                    kind = ExtensionKind::Theme;
                } else if !content_scripts.is_empty() {
                    kind = ExtensionKind::ContentScript;
                }
            }

            list.push(ExtensionInfo {
                id: file_name,
                name,
                version,
                description,
                path,
                kind,
                enabled,
                theme_colors,
                content_scripts,
            });
        } else if file_name.ends_with(".user.js") || file_name.ends_with(".js") {
            let matches = parse_userscript_matches(&path);
            list.push(ExtensionInfo {
                id: file_name.clone(),
                name: file_name,
                version: None,
                description: Some("Userscript (.user.js)".to_string()),
                content_scripts: vec![ContentScriptRule {
                    matches,
                    exclude_matches: Vec::new(),
                    js_files: vec![path.clone()],
                    css_files: Vec::new(),
                }],
                path,
                kind: ExtensionKind::Userscript,
                enabled,
                theme_colors: None,
            });
        }
    }

    list.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    list
}

fn match_pattern_matches(pattern: &str, url: &Url) -> bool {
    if pattern == "<all_urls>" || pattern == "*" {
        return matches!(url.scheme(), "http" | "https");
    }
    let Some((pat_scheme, rest)) = pattern.split_once("://") else {
        return wildcard_match(pattern, url.as_str());
    };
    if pat_scheme != "*" && pat_scheme != url.scheme() {
        return false;
    }
    let Some((pat_host, pat_path)) = rest.split_once('/') else {
        return false;
    };
    let Some(url_host) = url.host_str() else {
        return false;
    };
    let host_ok = if pat_host == "*" {
        true
    } else if let Some(sub) = pat_host.strip_prefix("*.") {
        url_host == sub || url_host.ends_with(&format!(".{sub}"))
    } else {
        url_host == pat_host
    };
    if !host_ok {
        return false;
    }
    let url_path = url.path().trim_start_matches('/');
    wildcard_match(pat_path, url_path)
}

fn wildcard_match(pattern: &str, text: &str) -> bool {
    let parts: Vec<&str> = pattern.split('*').collect();
    if parts.len() == 1 {
        return pattern == text;
    }
    let mut remainder = text;
    for (idx, part) in parts.iter().enumerate() {
        if part.is_empty() {
            continue;
        }
        if idx == 0 {
            if let Some(rest) = remainder.strip_prefix(part) {
                remainder = rest;
            } else {
                return false;
            }
        } else if idx == parts.len() - 1 && !pattern.ends_with('*') {
            return remainder.ends_with(part);
        } else if let Some(pos) = remainder.find(part) {
            remainder = &remainder[pos + part.len()..];
        } else {
            return false;
        }
    }
    true
}

pub(crate) fn build_injection_script_for_url(
    url: &Url,
    lang: &str,
    content_scripts_enabled: bool,
) -> Option<String> {
    if !content_scripts_enabled || !matches!(url.scheme(), "http" | "https") {
        return None;
    }

    let extensions = list_installed_extensions(lang);
    let mut chunks = Vec::new();

    for ext in extensions {
        if !ext.enabled || ext.content_scripts.is_empty() {
            continue;
        }
        for rule in &ext.content_scripts {
            let matched = rule.matches.iter().any(|p| match_pattern_matches(p, url));
            let excluded = rule
                .exclude_matches
                .iter()
                .any(|p| match_pattern_matches(p, url));
            if !matched || excluded {
                continue;
            }

            for css_path in &rule.css_files {
                if let Ok(css) = fs::read_to_string(css_path) {
                    let encoded = serde_json::to_string(&css).unwrap_or_default();
                    let ext_id_json = serde_json::to_string(&ext.id).unwrap_or_default();
                    chunks.push(format!(
                        "(function(){{try{{var s=document.createElement('style');s.setAttribute('data-librenav-ext',{ext_id_json});s.textContent={encoded};(document.head||document.documentElement).appendChild(s);}}catch(e){{}}}})();"
                    ));
                }
            }

            for js_path in &rule.js_files {
                if let Ok(js) = fs::read_to_string(js_path) {
                    chunks.push(format!(
                        "(function(){{try{{\n{js}\n}}catch(e){{console.warn('[LibreNav ext]',e);}}}})();"
                    ));
                }
            }
        }
    }

    if chunks.is_empty() {
        return None;
    }

    // Minimal WebExtension compatibility shim for content scripts.
    let shim = r#"
(function(){
  if (!window.chrome) window.chrome = {};
  if (!window.chrome.runtime) {
    window.chrome.runtime = {
      id: 'librenav-runtime',
      getURL: function(p) { return p; },
      sendMessage: function(_m, cb) { if (typeof cb === 'function') cb(); },
      onMessage: { addListener: function() {}, removeListener: function() {} }
    };
  }
  if (!window.chrome.storage) {
    window.chrome.storage = {
      local: {
        get: function(keys, cb) {
          var out = {};
          if (typeof keys === 'string') {
            try { out[keys] = JSON.parse(localStorage.getItem('ln_ext_' + keys)); } catch(_) {}
          }
          if (typeof cb === 'function') cb(out);
          return Promise.resolve(out);
        },
        set: function(obj, cb) {
          for (var k in obj) {
            try { localStorage.setItem('ln_ext_' + k, JSON.stringify(obj[k])); } catch(_) {}
          }
          if (typeof cb === 'function') cb();
          return Promise.resolve();
        }
      }
    };
  }
  if (!window.browser) window.browser = window.chrome;
})();
"#;

    Some(format!("{shim}\n{}", chunks.join("\n")))
}

pub(crate) fn find_ext_id(input: &str) -> Result<String, String> {
    let trimmed = input.trim();
    if trimmed.len() == 32 && trimmed.chars().all(|c| c.is_ascii_lowercase()) {
        return Ok(trimmed.to_string());
    }
    if let Some(pos) = trimmed.find("/detail/") {
        let rest = &trimmed[pos + 8..];
        let parts: Vec<&str> = rest.split('/').collect();
        for part in parts {
            let clean = part.split('?').next().unwrap_or(part);
            if clean.len() == 32 && clean.chars().all(|c| c.is_ascii_lowercase()) {
                return Ok(clean.to_string());
            }
        }
    }
    Err("ID d'extension Chrome invalide (32 lettres minuscules ou URL Chrome Web Store attendue)."
        .into())
}

pub(crate) fn extract_amo_slug(input: &str) -> Result<String, String> {
    let trimmed = input.trim();
    if let Some(pos) = trimmed.find("/addon/") {
        let rest = &trimmed[pos + 7..];
        let slug = rest.split('/').next().unwrap_or(rest);
        let clean = slug.split('?').next().unwrap_or(slug);
        if !clean.is_empty() {
            return Ok(clean.to_string());
        }
    }
    let clean = trimmed.split('?').next().unwrap_or(trimmed);
    if !clean.contains('/') && !clean.contains('\\') && !clean.is_empty() {
        return Ok(clean.to_string());
    }
    Err("Identifiant ou URL Firefox Add-ons non reconnu.".into())
}

pub(crate) fn install_chrome_extension(id_or_url: &str, lang: &str) -> Result<String, String> {
    let ext_id = find_ext_id(id_or_url)?;
    let Some(dest_dir) = extensions_directory_path() else {
        return Err("Dossier des extensions introuvable.".into());
    };
    let _ = fs::create_dir_all(&dest_dir);

    let crx_url = format!(
        "https://clients2.google.com/service/update2/crx?response=redirect&prod=chromecrx&prodchannel=&prodversion=141.0.7390.55&lang=en-US&acceptformat=crx3,puff&x=id%3D{ext_id}%26installsource%3Dondemand%26uc"
    );

    let temp_crx = std::env::temp_dir().join(format!("{ext_id}.crx"));
    let temp_zip = std::env::temp_dir().join(format!("{ext_id}.zip"));

    let curl_status = std::process::Command::new("curl")
        .args([
            "-sSL",
            "-A",
            "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36",
            "-o",
        ])
        .arg(&temp_crx)
        .arg(&crx_url)
        .status()
        .map_err(|e| format!("curl a échoué : {e}"))?;

    if !curl_status.success() || !temp_crx.exists() {
        return Err("Téléchargement du fichier CRX échoué.".into());
    }

    let crx_bytes = fs::read(&temp_crx).map_err(|e| format!("Lecture CRX échouée : {e}"))?;
    let _ = fs::remove_file(&temp_crx);

    let pk_magic = [0x50, 0x4b, 0x03, 0x04];
    let Some(offset) = crx_bytes.windows(4).position(|w| w == pk_magic) else {
        return Err("Fichier CRX invalide (en-tête ZIP introuvable).".into());
    };

    fs::write(&temp_zip, &crx_bytes[offset..])
        .map_err(|e| format!("Écriture ZIP échouée : {e}"))?;

    let ext_target_dir = dest_dir.join(&ext_id);
    let _ = fs::create_dir_all(&ext_target_dir);

    let unzip_status = std::process::Command::new("unzip")
        .args(["-q", "-o"])
        .arg(&temp_zip)
        .arg("-d")
        .arg(&ext_target_dir)
        .status();

    let _ = fs::remove_file(&temp_zip);

    if !unzip_status.map(|s| s.success()).unwrap_or(false) {
        return Err("Extraction de l'archive de l'extension échouée.".into());
    }

    let (name, ver) = read_manifest_summary(&ext_target_dir, &ext_id, lang);
    Ok(format!("Extension '{name}' (v{ver}) installée avec succès !"))
}

pub(crate) fn install_firefox_extension(slug_or_url: &str, lang: &str) -> Result<String, String> {
    let slug = extract_amo_slug(slug_or_url)?;
    let Some(dest_dir) = extensions_directory_path() else {
        return Err("Dossier des extensions introuvable.".into());
    };
    let _ = fs::create_dir_all(&dest_dir);

    let xpi_url = format!("https://addons.mozilla.org/firefox/downloads/latest/{slug}/latest.xpi");
    let temp_xpi = std::env::temp_dir().join(format!("{slug}.xpi"));

    let curl_status = std::process::Command::new("curl")
        .args([
            "-sSL",
            "-A",
            "Mozilla/5.0 (X11; Linux x86_64; rv:130.0) Gecko/20100101 Firefox/130.0",
            "-o",
        ])
        .arg(&temp_xpi)
        .arg(&xpi_url)
        .status()
        .map_err(|e| format!("curl a échoué : {e}"))?;

    if !curl_status.success() || !temp_xpi.exists() {
        return Err("Téléchargement du module Firefox échoué.".into());
    }

    let ext_target_dir = dest_dir.join(&slug);
    let _ = fs::create_dir_all(&ext_target_dir);

    let unzip_status = std::process::Command::new("unzip")
        .args(["-q", "-o"])
        .arg(&temp_xpi)
        .arg("-d")
        .arg(&ext_target_dir)
        .status();

    let _ = fs::remove_file(&temp_xpi);

    if !unzip_status.map(|s| s.success()).unwrap_or(false) {
        return Err("Extraction de l'archive XPI échouée.".into());
    }

    let (name, ver) = read_manifest_summary(&ext_target_dir, &slug, lang);
    Ok(format!(
        "Extension Firefox '{name}' (v{ver}) installée avec succès !"
    ))
}

pub(crate) fn install_local_extension(path_str: &str) -> Result<String, String> {
    let trimmed = path_str.trim().trim_matches('\'').trim_matches('"');
    let src = Path::new(trimmed);
    if !src.exists() {
        return Err(format!("Le fichier '{}' n'existe pas.", src.display()));
    }
    let Some(dest_dir) = extensions_directory_path() else {
        return Err("Dossier des extensions introuvable.".into());
    };
    let _ = fs::create_dir_all(&dest_dir);
    let file_name = src
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    if file_name.ends_with(".user.js") || file_name.ends_with(".js") {
        let dest = dest_dir.join(&file_name);
        fs::copy(src, dest).map_err(|e| format!("Copie échouée : {e}"))?;
        return Ok(format!("Userscript '{file_name}' installé avec succès !"));
    }

    if file_name.ends_with(".crx") {
        let crx_bytes = fs::read(src).map_err(|e| format!("Lecture échouée : {e}"))?;
        let pk_magic = [0x50, 0x4b, 0x03, 0x04];
        let Some(offset) = crx_bytes.windows(4).position(|w| w == pk_magic) else {
            return Err("Format CRX invalide.".into());
        };
        let stem = src.file_stem().unwrap_or_default().to_string_lossy();
        let ext_target_dir = dest_dir.join(stem.as_ref());
        let _ = fs::create_dir_all(&ext_target_dir);
        let temp_zip = std::env::temp_dir().join("local_crx.zip");
        fs::write(&temp_zip, &crx_bytes[offset..]).map_err(|e| format!("{e}"))?;
        let _ = std::process::Command::new("unzip")
            .args(["-q", "-o"])
            .arg(&temp_zip)
            .arg("-d")
            .arg(&ext_target_dir)
            .status();
        let _ = fs::remove_file(&temp_zip);
        return Ok(format!("Extension '{stem}' installée depuis fichier CRX !"));
    }

    if file_name.ends_with(".xpi") || file_name.ends_with(".zip") {
        let stem = src.file_stem().unwrap_or_default().to_string_lossy();
        let ext_target_dir = dest_dir.join(stem.as_ref());
        let _ = fs::create_dir_all(&ext_target_dir);
        let status = std::process::Command::new("unzip")
            .args(["-q", "-o"])
            .arg(src)
            .arg("-d")
            .arg(&ext_target_dir)
            .status();
        return if status.map(|s| s.success()).unwrap_or(false) {
            Ok(format!("Archive '{stem}' extraite et installée !"))
        } else {
            Err("Extraction de l'archive échouée.".into())
        };
    }

    if src.is_dir() {
        let stem = src.file_name().unwrap_or_default().to_string_lossy();
        let ext_target_dir = dest_dir.join(stem.as_ref());
        let _ = fs::create_dir_all(&ext_target_dir);
        let _ = std::process::Command::new("cp")
            .args(["-r", "."])
            .current_dir(src)
            .arg(&ext_target_dir)
            .status();
        return Ok(format!("Dossier d'extension '{stem}' installé !"));
    }

    Err("Format non supporté (utilisez .crx, .xpi, .zip, .user.js ou un dossier).".into())
}

pub(crate) fn delete_extension(path: &Path) -> Result<(), String> {
    if path.is_dir() {
        fs::remove_dir_all(path).map_err(|e| format!("Suppression échouée : {e}"))
    } else {
        fs::remove_file(path).map_err(|e| format!("Suppression échouée : {e}"))
    }
}

fn read_manifest_summary(ext_dir: &Path, fallback_name: &str, lang: &str) -> (String, String) {
    let manifest_path = ext_dir.join("manifest.json");
    let mut name = fallback_name.to_string();
    let mut ver = String::from("1.0");
    if let Ok(content) = fs::read_to_string(&manifest_path)
        && let Ok(val) = serde_json::from_str::<serde_json::Value>(&content)
    {
        if let Some(n) = val.get("name").and_then(|v| v.as_str()) {
            name = resolve_localized_message(ext_dir, n, lang);
        }
        if let Some(v) = val.get("version").and_then(|v| v.as_str()) {
            ver = v.to_string();
        }
    }
    (name, ver)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adblock_blocks_trackers_but_allows_first_party() {
        let page = Url::parse("https://example.org/article").unwrap();
        let tracker = Url::parse("https://www.google-analytics.com/analytics.js").unwrap();
        let first_party = Url::parse("https://example.org/app.js").unwrap();
        assert!(should_block_request(&tracker, Some(&page)));
        assert!(!should_block_request(&first_party, Some(&page)));
    }

    #[test]
    fn test_match_pattern_wildcards() {
        let url = Url::parse("https://sub.example.org/path/page.html").unwrap();
        assert!(match_pattern_matches("<all_urls>", &url));
        assert!(match_pattern_matches("https://*.example.org/*", &url));
        assert!(!match_pattern_matches("https://other.org/*", &url));
    }
}
