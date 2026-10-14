# LibreNav

[Esperanto](README.md) · [Français](README.fr.md) · [English](README.en.md) · [Español](README.es.md)

LibreNav est un prototype libre de navigateur de bureau : une interface familière inspirée de Chromium, propulsée par le moteur Servo plutôt que Chromium. Aucun code fermé, logo ou service Google n’est repris.

## Démarrage rapide

```sh
./scripts/run-librenav.sh
```

Pour une première construction sur Fedora, installez les dépendances indiquées dans le [README principal](README.md), puis lancez `./mach build --dev --media-stack dummy`.

## Fonctionnalités actuelles

- Onglets et fenêtres multiples
- Omnibox pour adresses et recherches
- Navigation, rechargement et page d’accueil
- Favoris persistants stockés localement
- Interface sombre et accessible, proche des conventions Chromium
- Nouvel onglet local en espéranto, français, anglais et espagnol
- Aucun CDN, police distante ou ajout de télémétrie

## Limites

LibreNav n’est pas encore destiné à remplacer un navigateur mature. Les extensions, la synchronisation, l’impression et plusieurs fonctions avancées restent à construire. La compatibilité des sites dépend de l’état actuel de Servo.

## Validation

```sh
./mach fmt
./mach check --media-stack dummy -p servoshell
```

Le code Servo conserve sa licence MPL-2.0 et ses licences tierces. Les modifications LibreNav sont également sous MPL-2.0. LibreNav n’est pas affilié à Google.
