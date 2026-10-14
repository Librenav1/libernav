# LibreNav

[Esperanto](README.md) · [Français](README.fr.md) · [English](README.en.md) · [Español](README.es.md)

![Stato: prototipo](docs/badges/status.svg) ![Versio: 0.1](docs/badges/version.svg) ![Permesilo: MPL-2.0](docs/badges/license.svg) ![Motoro: Servo](docs/badges/engine.svg)

LibreNav estas libera labortabla retumilo kun konata Chromium-stila interfaco kaj tute alia motoro: Servo. Ĝi ne kopias fermitan kodon, markojn aŭ retajn servojn de Google.

## Rapida komenco

```sh
./scripts/run-librenav.sh
```

La unua konstruo bezonas la oficialajn Servo-dependaĵojn. En Fedora:

```sh
sudo dnf install cabextract dejavu-fonts-all expat-devel gperf libXmu-devel libjpeg-turbo-devel libxkbcommon-x11 llvm mesa-libEGL-devel mesa-libGL-devel mesa-vulkan-drivers ncurses-devel python3-devel
./mach bootstrap --yes
./mach build --dev --media-stack dummy
./scripts/run-librenav.sh
```

## Nunaj funkcioj

- Pluraj langetoj kaj fenestroj
- Omnibox por adresoj kaj serĉoj
- Navigado reen, antaŭen, reŝargo kaj hejmpaĝo
- Daŭraj legosignoj konservitaj loke
- Malluma, alirebla Chromium-stila aranĝo
- Loka novlangeta paĝo en Esperanto, franca, angla kaj hispana
- Neniu aldona telemetrio, ekstera CDN aŭ fora tiparo
- Servo por HTML, CSS, aranĝo kaj bildigo

## Limigoj de la prototipo

LibreNav ankoraŭ ne estas ĉiutaga anstataŭanto por maturaj retumiloj. Etendaĵoj, sinkronigo, kompleta elŝuta administrado kaj presado ne estas pretaj. La kongrueco de retejoj dependas de la nuna stato de Servo.

## Arkitekturo

```text
ports/servoshell/desktop/gui.rs       Chromium-stila denaska interfaco per egui
ports/servoshell/desktop/             fenestroj, enigo kaj Servo-integriĝo
components/servo/                     enkonstruiga API de la Servo-motoro
resources/resource_protocol/          lokaj internaj paĝoj kaj i18n
resources/librenav.svg                    propra vektora identeco
scripts/run-librenav.sh                   loka lanĉilo
```

## Sekureco kaj privateco

La interna novlangeta paĝo estas memstara kaj ŝargas neniun trian rimedon. Serĉo kontaktas DuckDuckGo nur post eksplicita sendo de la formularo. Servo estas prototipa motoro; uzu LibreNav por disvolvado kaj testado ĝis sekureca revizio kaj pli vasta kongrueca validigo estos kompletigitaj.

## Kontribuo

Kreu celitan branĉon, konservu ŝanĝojn malgrandaj, rulu `./mach fmt` kaj `./mach check --media-stack dummy -p servoshell`, kaj dokumentu observeblajn kondutŝanĝojn. Ne aldonu proprietajn komponantojn, telemetrion aŭ eksterajn CDN-ojn.

## Permesilo

La Servo-kodo restas sub MPL-2.0 kaj la respektivaj triaj permesiloj. La LibreNav-ŝanĝoj en ĉi tiu branĉo estas ofertataj sub MPL-2.0. LibreNav ne estas aligita kun Google kaj ne uzas la markon aŭ emblemon Chrome.
