/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::RefCell;
use std::collections::{HashSet, VecDeque};
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use adblock::Engine;
use adblock::lists::{FilterSet, ParseOptions};
use adblock::request::Request;
use url::Url;

use crate::desktop::settings::{BrowserSettings, profile_file_path};

pub(crate) struct FilterListDef {
    pub(crate) id: &'static str,
    pub(crate) name: &'static str,
    pub(crate) url: &'static str,
}

pub(crate) const OFFICIAL_FILTER_LISTS: &[FilterListDef] = &[
    FilterListDef {
        id: "easylist",
        name: "EasyList",
        url: "https://easylist.to/easylist/easylist.txt",
    },
    FilterListDef {
        id: "easyprivacy",
        name: "EasyPrivacy",
        url: "https://easylist.to/easylist/easyprivacy.txt",
    },
    FilterListDef {
        id: "ublock-filters",
        name: "uBlock Origin Filters",
        url: "https://ublockorigin.github.io/uAssets/filters/filters.txt",
    },
];

#[derive(Clone, Debug)]
pub(crate) struct FilterListStatus {
    pub(crate) id: &'static str,
    pub(crate) name: &'static str,
    pub(crate) installed: bool,
    pub(crate) line_count: usize,
}

struct AdblockRuntime {
    engine: Engine,
    enabled: bool,
    disabled_hosts: HashSet<String>,
    recent_blocked: VecDeque<String>,
}

thread_local! {
    static ADBLOCK_RUNTIME: RefCell<AdblockRuntime> = RefCell::new(AdblockRuntime {
        engine: build_engine_from_disk(),
        enabled: true,
        disabled_hosts: HashSet::new(),
        recent_blocked: VecDeque::with_capacity(32),
    });
}
static ADBLOCK_BLOCKED_COUNT: AtomicU64 = AtomicU64::new(0);

const BASELINE_ABP_RULES: &str = r#"
||doubleclick.net^
||googlesyndication.com^
||googleadservices.com^
||google-analytics.com^
||googletagmanager.com^
||googletagservices.com^
||pagead2.googlesyndication.com^
||adservice.google.com^
||connect.facebook.net^
||pixel.facebook.com^
||analytics.twitter.com^
||ads-twitter.com^
||taboola.com^
||outbrain.com^
||criteo.com^
||criteo.net^
||adnxs.com^
||pubmatic.com^
||rubiconproject.com^
||openx.net^
||casalemedia.com^
||scorecardresearch.com^
||quantserve.com^
||hotjar.com^
||mixpanel.com^
||segment.io^
||segment.com^
||amplitude.com^
||clarity.ms^
||mc.yandex.ru^
||amazon-adsystem.com^
||moatads.com^
||bidswitch.net^
||smartadserver.com^
||teads.tv^
||adsrvr.org^
||demdex.net^
||omtrdc.net^
||chartbeat.com^
||newrelic.com^
||nr-data.net^
||bugsnag.com^
"#;

fn filter_lists_dir() -> Option<PathBuf> {
    profile_file_path("filter-lists")
}

fn custom_filters_path() -> Option<PathBuf> {
    profile_file_path("custom-filters.txt")
}

fn build_engine_from_disk() -> Engine {
    let mut set = FilterSet::new(false);
    set.add_filter_list(BASELINE_ABP_RULES.to_string(), ParseOptions::default());

    if let Some(dir) = filter_lists_dir() {
        for def in OFFICIAL_FILTER_LISTS {
            let path = dir.join(format!("{}.txt", def.id));
            if let Ok(content) = fs::read_to_string(&path) {
                set.add_filter_list(content, ParseOptions::default());
            }
        }
    }

    if let Some(custom_path) = custom_filters_path()
        && let Ok(custom) = fs::read_to_string(&custom_path)
    {
        set.add_filter_list(custom, ParseOptions::default());
    }

    Engine::new_with_filter_set(set)
}

pub(crate) fn sync_settings(settings: &BrowserSettings) {
    ADBLOCK_RUNTIME.with(|rt| {
        let mut guard = rt.borrow_mut();
        guard.enabled = settings.ublock_enabled;
        guard.disabled_hosts = settings.ublock_disabled_hosts.clone();
    });
}

pub(crate) fn reload_engine() {
    let new_engine = build_engine_from_disk();
    ADBLOCK_RUNTIME.with(|rt| {
        rt.borrow_mut().engine = new_engine;
    });
}

pub(crate) fn blocked_requests_count() -> u64 {
    ADBLOCK_BLOCKED_COUNT.load(Ordering::Relaxed)
}

pub(crate) fn recent_blocked_urls() -> Vec<String> {
    ADBLOCK_RUNTIME.with(|rt| rt.borrow().recent_blocked.iter().cloned().collect())
}

pub(crate) fn list_filter_lists() -> Vec<FilterListStatus> {
    let dir = filter_lists_dir();
    OFFICIAL_FILTER_LISTS
        .iter()
        .map(|def| {
            let path = dir.as_ref().map(|d| d.join(format!("{}.txt", def.id)));
            let line_count = path
                .as_ref()
                .and_then(|p| fs::read_to_string(p).ok())
                .map(|s| s.lines().filter(|l| !l.trim().is_empty() && !l.starts_with('!')).count())
                .unwrap_or(0);
            FilterListStatus {
                id: def.id,
                name: def.name,
                installed: path.is_some_and(|p| p.is_file()),
                line_count,
            }
        })
        .collect()
}

pub(crate) fn download_filter_list(id: &str) -> Result<String, String> {
    let Some(def) = OFFICIAL_FILTER_LISTS.iter().find(|d| d.id == id) else {
        return Err(format!("Liste inconnue : {id}"));
    };
    let Some(dir) = filter_lists_dir() else {
        return Err("Dossier de profil introuvable.".into());
    };
    fs::create_dir_all(&dir).map_err(|e| format!("Création dossier échouée : {e}"))?;
    let dest = dir.join(format!("{}.txt", def.id));

    let status = std::process::Command::new("curl")
        .args(["-sSL", "--max-time", "30", "-o"])
        .arg(&dest)
        .arg(def.url)
        .status()
        .map_err(|e| format!("curl a échoué : {e}"))?;

    if !status.success() || !dest.exists() {
        return Err(format!("Téléchargement de {} échoué.", def.name));
    }

    let content = fs::read_to_string(&dest).map_err(|e| format!("Lecture échouée : {e}"))?;
    let rules = content
        .lines()
        .filter(|l| !l.trim().is_empty() && !l.starts_with('!'))
        .count();
    if rules < 10 {
        let _ = fs::remove_file(&dest);
        return Err(format!("Liste {} invalide ou vide.", def.name));
    }

    reload_engine();
    Ok(format!("{} mise à jour ({rules} règles actives).", def.name))
}

#[allow(dead_code)]
pub(crate) fn load_custom_filters() -> String {
    custom_filters_path()
        .and_then(|p| fs::read_to_string(p).ok())
        .unwrap_or_default()
}

#[allow(dead_code)]
pub(crate) fn save_custom_filters(text: &str) -> Result<(), String> {
    let Some(path) = custom_filters_path() else {
        return Err("Dossier de profil introuvable.".into());
    };
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    fs::write(&path, text).map_err(|e| format!("Écriture échouée : {e}"))?;
    reload_engine();
    Ok(())
}

pub(crate) fn check_request(request_url: &Url, page_url: Option<&Url>) -> bool {
    let scheme = request_url.scheme();
    if scheme != "http" && scheme != "https" {
        return false;
    }

    ADBLOCK_RUNTIME.with(|rt| {
        let mut guard = rt.borrow_mut();
        if !guard.enabled {
            return false;
        }
        if let Some(page_host) = page_url.and_then(|u| u.host_str())
            && guard.disabled_hosts.contains(page_host)
        {
            return false;
        }

        let source_url = page_url.map(|u| u.as_str()).unwrap_or("");
        let Ok(req) = Request::new(request_url.as_str(), source_url, "other", "") else {
            return false;
        };
        let result = guard.engine.check_network_request(&req);
        if result.filter.is_some() {
            ADBLOCK_BLOCKED_COUNT.fetch_add(1, Ordering::Relaxed);
            if guard.recent_blocked.len() >= 30 {
                guard.recent_blocked.pop_back();
            }
            guard.recent_blocked.push_front(request_url.to_string());
            return true;
        }
        false
    })
}

pub(crate) fn cosmetic_script_for_url(page_url: &Url) -> Option<String> {
    if !matches!(page_url.scheme(), "http" | "https") {
        return None;
    }
    ADBLOCK_RUNTIME.with(|rt| {
        let guard = rt.borrow();
        if !guard.enabled {
            return None;
        }
        if let Some(page_host) = page_url.host_str()
            && guard.disabled_hosts.contains(page_host)
        {
            return None;
        }

        let cosmetic = guard.engine.url_cosmetic_resources(page_url.as_str());
        let mut chunks = Vec::new();
        if !cosmetic.hide_selectors.is_empty() {
            let selectors: Vec<String> = cosmetic.hide_selectors.into_iter().take(400).collect();
            let css = format!("{} {{ display: none !important; }}", selectors.join(", "));
            let encoded = serde_json::to_string(&css).unwrap_or_default();
            chunks.push(format!(
                "(function(){{try{{var s=document.createElement('style');s.setAttribute('data-librenav-adblock','1');s.textContent={encoded};(document.head||document.documentElement).appendChild(s);}}catch(e){{}}}})();"
            ));
        }
        if !cosmetic.injected_script.is_empty() {
            chunks.push(format!(
                "(function(){{try{{\n{}\n}}catch(e){{}}}})();",
                cosmetic.injected_script
            ));
        }

        if chunks.is_empty() {
            None
        } else {
            Some(chunks.join("\n"))
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adblock_engine_blocks_tracker_and_allows_first_party() {
        let page = Url::parse("https://example.org/news").unwrap();
        let ad = Url::parse("https://doubleclick.net/ads.js").unwrap();
        let own = Url::parse("https://example.org/main.js").unwrap();
        assert!(check_request(&ad, Some(&page)));
        assert!(!check_request(&own, Some(&page)));
    }
}
