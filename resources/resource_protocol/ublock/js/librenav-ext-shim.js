/**
 * LibreNav Native Extension Runtime Shim for uBlock Origin
 * Provides official WebExtension APIs (chrome.i18n, chrome.runtime, chrome.storage)
 * so official extension pages (popup-fenix.html, dashboard.html) run natively in Servo.
 */
(function() {
    'use strict';

    // Parse language from URL parameter or navigator
    const urlParams = new URLSearchParams(window.location.search);
    const lang = urlParams.get('lang') || (navigator.language || 'fr').slice(0, 2);

    // Multilingual messages table for core UI elements
    const locales = {
        fr: {
            "extName": "uBlock Origin",
            "extShortDesc": "Un bloqueur de nuisances efficace et sobre en ressources.",
            "dashboardName": "uBlock₀ — Tableau de bord",
            "settingsPageName": "Paramètres",
            "3pPageName": "Listes de filtres",
            "1pPageName": "Mes filtres",
            "rulesPageName": "Filtrage dynamique",
            "aboutPageName": "À propos",
            "popupTipSaveRules": "Enregistrer les modifications",
            "popupTipRevertRules": "Annuler les modifications",
            "popupNoPopups_v2": "Bloquer les fenêtres pop-up",
            "popupNoLargeMedia_v2": "Bloquer les médias volumineux",
            "popupNoCosmeticFiltering_v2": "Filtrage cosmétique",
            "popupNoRemoteFonts_v2": "Bloquer les polices distantes",
            "popupNoScripting_v2": "Désactiver JavaScript",
            "popupBlockedOnThisPage_v2": "Requêtes bloquées sur cette page",
            "popupDomainsConnected_v2": "Domaines connectés",
            "popupBlockedSinceInstall_v2": "Requêtes bloquées au total",
            "popupTipZapper": "Mode zappeur d'éléments",
            "popupTipPicker": "Sélecteur d'éléments",
            "popupTipReport": "Signaler un problème sur ce site",
            "popupTipLog": "Ouvrir le journal",
            "popupTipDashboard": "Ouvrir le tableau de bord",
            "popupVersion": "Version",
            "popupMoreButton_v2": "Plus d'outils",
            "popupLessButton_v2": "Moins d'outils",
            "popupAnyRulePrompt": "Tous",
            "popup3pScriptFilter": "Scripts tiers",
            "popup3pFrameFilter": "Cadres tiers",
            "popupImageRulePrompt": "Images",
            "popup3pAnyRulePrompt": "Tiers",
            "popupInlineScriptRulePrompt": "Scripts intégrés",
            "popup1pScriptRulePrompt": "Scripts 1re partie",
            "popup3pScriptRulePrompt": "Scripts tiers",
            "popup3pFrameRulePrompt": "Cadres tiers",
            "popupHitDomainCount": "Domaines connectés",
            "popupBlockedStats": "Requêtes bloquées",
            "popupPowerSwitchInfo": "Cliquer pour activer/désactiver le filtrage pour ce site",
            "popupTipGlobalRules": "Règles globales",
            "popupTipLocalRules": "Règles locales",
            "loggerRowFiltererBuiltinNot": "non",
            "loggerRowFiltererBuiltinBlocked": "bloqué",
            "loggerRowFiltererBuiltinAllowed": "autorisé",
            "unprocessedRequestTooltip": "Certaines requêtes n'ont pas pu être filtrées",
            "1pFormatHint": "Ajoutez vos règles de filtrage personnalisées ci-dessous :"
        },
        eo: {
            "extName": "uBlock Origin",
            "extShortDesc": "Efikaj kaj rimed-ŝparemaj ĝenoblokiloj por LibreNav.",
            "dashboardName": "uBlock₀ — Stirpanelo",
            "settingsPageName": "Agordoj",
            "3pPageName": "Filtrilolistoj",
            "1pPageName": "Miaj filtriloj",
            "rulesPageName": "Dinamika filtrado",
            "aboutPageName": "Pri",
            "popupTipSaveRules": "Konservi regulojn",
            "popupTipRevertRules": "Malfari regulojn",
            "popupNoPopups_v2": "Bloki ŝprucfenestrojn",
            "popupNoLargeMedia_v2": "Bloki grandajn aŭdvidaĵojn",
            "popupNoCosmeticFiltering_v2": "Kosmetika filtrado",
            "popupNoRemoteFonts_v2": "Bloki forajn tiparojn",
            "popupNoScripting_v2": "Malŝalti JavaScript",
            "popupBlockedOnThisPage_v2": "Blokitaj sur tiu ĉi paĝo",
            "popupDomainsConnected_v2": "Konektitaj domajnoj",
            "popupBlockedSinceInstall_v2": "Entute blokitaj",
            "popupTipZapper": "Element-forigilo",
            "popupTipPicker": "Element-elektilo",
            "popupTipReport": "Raporti problemon",
            "popupTipLog": "Malfermi protokolon",
            "popupTipDashboard": "Malfermi stirpanelon",
            "popupVersion": "Versio",
            "popupMoreButton_v2": "Pli da iloj",
            "popupLessButton_v2": "Malpli",
            "popupAnyRulePrompt": "Ĉiuj",
            "popup3pScriptFilter": "Triapartaj skriptoj",
            "popup3pFrameFilter": "Triapartaj kadroj",
            "popupImageRulePrompt": "Bildoj",
            "popup3pAnyRulePrompt": "Triapartaj",
            "popupInlineScriptRulePrompt": "Enkonstruitaj skriptoj",
            "popup1pScriptRulePrompt": "Unupartiaj skriptoj",
            "popup3pScriptRulePrompt": "Triapartaj skriptoj",
            "popup3pFrameRulePrompt": "Triapartaj kadroj",
            "popupHitDomainCount": "Konektitaj domajnoj",
            "popupBlockedStats": "Blokitaj petoj",
            "popupPowerSwitchInfo": "Alklaku por ŝalti aŭ malŝalti filtradon por tiu ĉi retejo",
            "popupTipGlobalRules": "Ĝeneralaj reguloj",
            "popupTipLocalRules": "Lokaj reguloj"
        },
        en: {
            "extName": "uBlock Origin",
            "extShortDesc": "An efficient and lightweight content blocker.",
            "dashboardName": "uBlock₀ — Dashboard",
            "settingsPageName": "Settings",
            "3pPageName": "Filter lists",
            "1pPageName": "My filters",
            "rulesPageName": "My rules",
            "aboutPageName": "About",
            "popupTipSaveRules": "Save changes",
            "popupTipRevertRules": "Revert changes",
            "popupNoPopups_v2": "Block all popups",
            "popupNoLargeMedia_v2": "Block large media elements",
            "popupNoCosmeticFiltering_v2": "Cosmetic filtering",
            "popupNoRemoteFonts_v2": "Block remote fonts",
            "popupNoScripting_v2": "Disable JavaScript",
            "popupBlockedOnThisPage_v2": "Requests blocked on this page",
            "popupDomainsConnected_v2": "Connected domains",
            "popupBlockedSinceInstall_v2": "Total requests blocked",
            "popupTipZapper": "Element zapper mode",
            "popupTipPicker": "Element picker mode",
            "popupTipReport": "Report an issue on this page",
            "popupTipLog": "Open logger",
            "popupTipDashboard": "Open dashboard",
            "popupVersion": "Version",
            "popupMoreButton_v2": "More",
            "popupLessButton_v2": "Less",
            "popupAnyRulePrompt": "All",
            "popup3pScriptFilter": "3rd-party scripts",
            "popup3pFrameFilter": "3rd-party frames",
            "popupImageRulePrompt": "Images",
            "popup3pAnyRulePrompt": "3rd-party",
            "popupInlineScriptRulePrompt": "Inline scripts",
            "popup1pScriptRulePrompt": "1st-party scripts",
            "popup3pScriptRulePrompt": "3rd-party scripts",
            "popup3pFrameRulePrompt": "3rd-party frames",
            "popupHitDomainCount": "Domains connected",
            "popupBlockedStats": "Requests blocked",
            "popupPowerSwitchInfo": "Click to toggle filtering for this site",
            "popupTipGlobalRules": "Global rules",
            "popupTipLocalRules": "Local rules"
        },
        es: {
            "extName": "uBlock Origin",
            "extShortDesc": "Un bloqueador de contenido eficiente y ligero.",
            "dashboardName": "uBlock₀ — Panel de control",
            "settingsPageName": "Configuración",
            "3pPageName": "Listas de filtros",
            "1pPageName": "Mis filtros",
            "rulesPageName": "Filtrado dinámico",
            "aboutPageName": "Acerca de",
            "popupTipSaveRules": "Guardar cambios",
            "popupTipRevertRules": "Revertir cambios",
            "popupNoPopups_v2": "Bloquear ventanas emergentes",
            "popupNoLargeMedia_v2": "Bloquear elementos multimedia grandes",
            "popupNoCosmeticFiltering_v2": "Filtrado cosmético",
            "popupNoRemoteFonts_v2": "Bloquear fuentes remotas",
            "popupNoScripting_v2": "Desactivar JavaScript",
            "popupBlockedOnThisPage_v2": "Solicitudes bloqueadas en esta página",
            "popupDomainsConnected_v2": "Dominios conectados",
            "popupBlockedSinceInstall_v2": "Total de solicitudes bloqueadas",
            "popupTipZapper": "Modo eliminador de elementos",
            "popupTipPicker": "Modo selector de elementos",
            "popupTipReport": "Informar de un problema",
            "popupTipLog": "Abrir registro",
            "popupTipDashboard": "Abrir panel de control",
            "popupVersion": "Versión",
            "popupMoreButton_v2": "Más",
            "popupLessButton_v2": "Menos",
            "popupAnyRulePrompt": "Todos",
            "popup3pScriptFilter": "Scripts de terceros",
            "popup3pFrameFilter": "Marcos de terceros",
            "popupImageRulePrompt": "Imágenes",
            "popup3pAnyRulePrompt": "Terceros",
            "popupInlineScriptRulePrompt": "Scripts incrustados",
            "popup1pScriptRulePrompt": "Scripts de origen",
            "popup3pScriptRulePrompt": "Scripts de terceros",
            "popup3pFrameRulePrompt": "Marcos de terceros",
            "popupHitDomainCount": "Dominios conectados",
            "popupBlockedStats": "Solicitudes bloqueadas",
            "popupPowerSwitchInfo": "Haga clic para alternar el filtrado en este sitio",
            "popupTipGlobalRules": "Reglas globales",
            "popupTipLocalRules": "Reglas locales"
        }
    };

    const activeLocale = locales[lang] || locales.fr;

    // Synchronous message resolver
    function getMessage(key) {
        if (key === '@@ui_locale') return lang;
        if (activeLocale[key]) return activeLocale[key];
        if (locales.fr[key]) return locales.fr[key];
        if (locales.en[key]) return locales.en[key];
        return key;
    }

    // Default mock popup data for active website
    let isEnabled = true;
    try {
        const stored = localStorage.getItem('librenav_ublock_enabled');
        if (stored !== null) isEnabled = stored === 'true';
    } catch (_) {}

    const popupState = {
        pageHostname: 'revsearch.revlibertaire.com',
        pageDomain: 'revlibertaire.com',
        rawURL: 'https://revsearch.revlibertaire.com/',
        tabId: 1,
        netFilteringSwitch: isEnabled,
        firewallPaneMinimized: false,
        popupPanelSections: 15,
        matrixIsDirty: false,
        contentLastModified: Date.now(),
        hostnameDict: {
            'revsearch.revlibertaire.com': { domain: 'revlibertaire.com', counts: [0, 0, 0] }
        },
        firewallRules: {},
        cnameMap: []
    };

    // Virtual Extension Port for messaging
    class VirtualPort {
        constructor() {
            this.name = 'ublock-virtual-port';
            this.listeners = [];
            this.disconnectListeners = [];
            this.onMessage = {
                addListener: (fn) => this.listeners.push(fn),
                removeListener: (fn) => {
                    const idx = this.listeners.indexOf(fn);
                    if (idx !== -1) this.listeners.splice(idx, 1);
                }
            };
            this.onDisconnect = {
                addListener: (fn) => this.disconnectListeners.push(fn),
                removeListener: (fn) => {
                    const idx = this.disconnectListeners.indexOf(fn);
                    if (idx !== -1) this.disconnectListeners.splice(idx, 1);
                }
            };
        }

        postMessage(msg) {
            setTimeout(() => {
                const response = this.handleMessage(msg.msg);
                for (const listener of this.listeners) {
                    listener({ msgId: msg.msgId, msg: response });
                }
            }, 0);
        }

        handleMessage(request) {
            if (!request) return null;
            const what = request.what;

            if (what === 'getPopupData') {
                return popupState;
            }
            if (what === 'hasPopupContentChanged') {
                return false;
            }
            if (what === 'toggleNetFilteringSwitch') {
                popupState.netFilteringSwitch = !popupState.netFilteringSwitch;
                try {
                    localStorage.setItem('librenav_ublock_enabled', popupState.netFilteringSwitch);
                } catch (_) {}
                return popupState;
            }
            if (what === 'toggleHostnameSwitch') {
                return popupState;
            }
            if (what === 'localStorage') {
                const fn = request.fn;
                const args = request.args || [];
                try {
                    if (fn === 'getItemAsync' || fn === 'getItem') {
                        return localStorage.getItem(args[0]);
                    }
                    if (fn === 'setItem') {
                        localStorage.setItem(args[0], args[1]);
                        return true;
                    }
                    if (fn === 'removeItem') {
                        localStorage.removeItem(args[0]);
                        return true;
                    }
                } catch (_) {}
                return null;
            }
            if (what === 'gotoURL') {
                if (request.details && request.details.url) {
                    window.location.href = request.details.url;
                }
                return true;
            }
            return {};
        }

        disconnect() {
            for (const fn of this.disconnectListeners) fn();
        }
    }

    const virtualPort = new VirtualPort();

    // Chrome / WebExtension API mock object
    const webextApi = {
        i18n: {
            getMessage: getMessage,
            getUILanguage: () => lang
        },
        runtime: {
            id: 'cjpalhdlnbpafiamejdnhcphjbkeiagm',
            lastError: null,
            connect: (details) => virtualPort,
            sendMessage: (msg, callback) => {
                const res = virtualPort.handleMessage(msg);
                if (callback) setTimeout(() => callback(res), 0);
                return Promise.resolve(res);
            },
            onMessage: {
                addListener: () => {},
                removeListener: () => {}
            }
        },
        storage: {
            local: {
                get: (keys, callback) => {
                    const result = {};
                    if (callback) setTimeout(() => callback(result), 0);
                    return Promise.resolve(result);
                },
                set: (items, callback) => {
                    if (callback) setTimeout(() => callback(), 0);
                    return Promise.resolve();
                },
                remove: (keys, callback) => {
                    if (callback) setTimeout(() => callback(), 0);
                    return Promise.resolve();
                }
            }
        },
        tabs: {
            query: (queryInfo, callback) => {
                const tabs = [{ id: 1, url: popupState.rawURL, active: true }];
                if (callback) setTimeout(() => callback(tabs), 0);
                return Promise.resolve(tabs);
            },
            reload: () => {
                window.location.reload();
            }
        }
    };

    self.chrome = webextApi;
    self.browser = webextApi;

    // Safety fallback: if body stays loading after 300ms, force display
    document.addEventListener('DOMContentLoaded', () => {
        setTimeout(() => {
            if (document.body && document.body.classList.contains('loading')) {
                document.body.classList.remove('loading');
            }
        }, 300);
    });
})();
