<div align="center">

<img src="resources/librenav.svg" alt="LibreNav" width="96" height="96">

# LibreNav

**Navigateur de bureau libre basé sur Servo — Version Alpha (en développement)**

[Esperanto](README.md) · [Français](README.fr.md) · [English](README.en.md) · [Español](README.es.md)

![Statut: alpha (dev)](docs/badges/status.svg) ![Version: 0.7](docs/badges/version.svg) ![Licence: MPL-2.0](docs/badges/license.svg) ![Moteur: Servo](docs/badges/engine.svg)

</div>

> [!WARNING]
> **Alpha / En cours de développement** : LibreNav est un logiciel en phase **alpha**. L'interface, le moteur de filtrage et la gestion des scripts évoluent activement, et la compatibilité web dépend de l'avancement du moteur Servo. Les extensions WebExtensions complètes (API Chrome/Firefox complètes), la synchronisation et l'impression ne sont pas encore prêtes pour un usage quotidien principal.

LibreNav est un navigateur libre sous licence **MPL-2.0**, basé sur le moteur **Servo**, sans télémétrie, sans CDN tiers ni service propriétaire.

---

## Fonctionnalités actuelles (Alpha)

- **Onglets et fenêtres multiples** avec interface native `egui`
- **Moteur de thèmes (`theme.rs`)** : thèmes prédéfinis (*Sombre*, *Clair*, *Contraste élevé*, *Nord*) et prise en charge des thèmes `manifest.json` (`theme.colors`)
- **Bouclier intégré (`adblock-rust`)** : blocage réseau et filtrage cosmétique CSS avec téléchargement des listes officielles (**EasyList**, **EasyPrivacy**, **uBlock Origin Filters**) et journal des requêtes bloquées
- **Userscripts et scripts de contenu (`extensions.rs`)** : installation depuis GreasyFork / `.user.js` / fichiers locaux avec runtime `GM_*` (`GM_addStyle`, `GM_getValue`, `GM_setValue`, `GM_xmlhttpRequest`) et `content_scripts` CSS/JS
- **Paramètres, Historique et Favoris** persistés localement (`surface.rs`, `settings.rs`)
- **Interface multilingue** : français, espéranto, anglais, espagnol
- **Moteur de recherche par défaut** : RevSearch (`revsearch.revlibertaire.com`)

## Démarrage rapide

```sh
cargo build --locked -p servoshell --bin servoshell
./target/debug/servoshell
```

Ou via le lanceur local :

```sh
./scripts/run-librenav.sh
```

Pour une première compilation sur Fedora, installez les dépendances système listées dans le [README principal](README.md), puis lancez `./mach bootstrap --yes` et `./mach build --dev --media-stack dummy`.

## Architecture (`ports/servoshell/desktop/`)

```text
ports/servoshell/desktop/
├── gui.rs          Barre d'onglets, barre d'outils et menus popups
├── surface.rs      Panneaux Paramètres, Extensions, Historique et Favoris
├── theme.rs        Moteur de thèmes et application à egui::Visuals
├── settings.rs     Persistance des paramètres, favoris, historique et i18n
├── adblocker.rs    Moteur adblock-rust (EasyList / EasyPrivacy / uBlock / CSS)
└── extensions.rs   Gestionnaire d'extensions, runtime GM_* et content_scripts
```

## Tests et vérification

```sh
cargo test --locked -p servoshell --lib desktop::
cargo check --locked -p servoshell
```

## Licence

Le code amont de Servo conserve sa licence MPL-2.0 et ses licences tierces. Les ajouts et modifications LibreNav sont publiés sous **MPL-2.0**.
