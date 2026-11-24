<div align="center">

<img src="resources/librenav.svg" alt="LibreNav" width="96" height="96">

# LibreNav

**Libera labortabla retumilo konstruita sur Servo — Versio Alpha (en aktiva disvolvado)**

[Esperanto](README.md) · [Français](README.fr.md) · [English](README.en.md) · [Español](README.es.md)

![Statut: alpha (dev)](docs/badges/status.svg) ![Versio: 0.7](docs/badges/version.svg) ![Permesilo: MPL-2.0](docs/badges/license.svg) ![Motoro: Servo](docs/badges/engine.svg)

</div>

> [!WARNING]
> **Alpha / En aktiva disvolvado** : LibreNav estas ankoraŭ en **alpha**-fazo. Pluraj funkcioj (kompleta WebExtensions-runtime, sinkronigo, presado, plena elŝuta administrado) ankoraŭ estas disvolvataj kaj la kongrueco dependas de la motoro Servo. Ne uzu ĝin kiel ĉiutagan ĉefan retumilon sen konscio pri ĉi tiuj limoj.

LibreNav estas libera labortabla retumilo sub permesilo **MPL-2.0**, konstruita sur la motoro **Servo** sen proprieta kodo, sen markoj de Google kaj sen telemetrio.

---

## Nunaj funkcioj (Alpha)

- **Pluraj langetoj kaj fenestroj** kun denaska interfaco en `egui`
- **Etosmotoro (`theme.rs`)** : antaŭdifinitaj etosoj (*Sombre*, *Clair*, *Contraste élevé*, *Nord*) kaj apliko de etosoj el `manifest.json` (`theme.colors`)
- **Enkonstruita enhavblokilo (`adblock-rust`)** : reta blokado kaj kosmetika CSS-filtrado kun oficialaj listoj (**EasyList**, **EasyPrivacy**, **uBlock Origin Filters**) kaj protokolo de blokitaj petoj
- **Uzantskriptoj kaj enhavskriptoj (`extensions.rs`)** : instalo el GreasyFork / `.user.js` / lokaj dosieroj kun runtime `GM_*` (`GM_addStyle`, `GM_getValue`, `GM_setValue`, `GM_xmlhttpRequest`) kaj `content_scripts` CSS/JS
- **Agordoj, Historio kaj Legosignoj** konservitaj loke (`surface.rs`, `settings.rs`)
- **Multlingva interfaco** : Esperanto, franca, angla kaj hispana
- **Defaŭlta serĉilo** : RevSearch (`revsearch.revlibertaire.com`), sen telemetrio

## Rapida komenco

```sh
cargo build --locked -p servoshell --bin servoshell
./target/debug/servoshell
```

Aŭ per la loka lanĉilo :

```sh
./scripts/run-librenav.sh
```

Por unua konstruo en Fedora :

```sh
sudo dnf install cabextract dejavu-fonts-all expat-devel gperf libXmu-devel libjpeg-turbo-devel libxkbcommon-x11 llvm mesa-libEGL-devel mesa-libGL-devel mesa-vulkan-drivers ncurses-devel python3-devel
./mach bootstrap --yes
./mach build --dev --media-stack dummy
```

## Arkitekturo de la interfaco (`ports/servoshell/desktop/`)

```text
ports/servoshell/desktop/
├── gui.rs          Ĉefa fenestro, langetoj, ilobreto kaj ŝprucfenestroj
├── surface.rs      Panoloj de Agordoj, Etendaĵoj, Historio kaj Legosignoj
├── theme.rs        Etosmotoro (antaŭdifinitaj etosoj kaj manifest.json)
├── settings.rs     Lokaj agordoj, legosignoj, historio kaj i18n
├── adblocker.rs    Blokilo bazita sur adblock-rust (EasyList / uBlock / CSS)
└── extensions.rs   Administrilo de etendaĵoj kaj runtime GM_* / content_scripts
```

## Kontrolo kaj testoj

```sh
cargo test --locked -p servoshell --lib desktop::
cargo check --locked -p servoshell
```

## Permesilo

La kodo de Servo restas sub MPL-2.0 kaj siaj respektivaj triapartaj permesiloj. La ŝanĝoj de LibreNav estas ofertataj sub **MPL-2.0**.
