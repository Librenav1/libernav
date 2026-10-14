# LibreNav

[Esperanto](README.md) · [Français](README.fr.md) · [English](README.en.md) · [Español](README.es.md)

LibreNav is a free desktop-browser prototype: a familiar Chromium-style interface powered by Servo instead of Chromium. It copies no closed Google code, branding, or services.

## Quick start

```sh
./scripts/run-librenav.sh
```

For the first Fedora build, install the dependencies listed in the [main README](README.md), then run `./mach build --dev --media-stack dummy`.

## Current features

- Multiple tabs and windows
- Address-and-search omnibox
- Back, forward, reload, and home controls
- Persistent bookmarks stored locally
- Accessible dark interface following Chromium conventions
- Offline new-tab page in Esperanto, French, English, and Spanish
- No CDN, remote font, or added telemetry

## Limits

LibreNav is not yet a daily replacement for a mature browser. Extensions, synchronization, printing, and several advanced features remain to be built. Website compatibility follows Servo’s current capabilities.

## Validation

```sh
./mach fmt
./mach check --media-stack dummy -p servoshell
```

Servo code retains MPL-2.0 and applicable third-party licenses. LibreNav changes are also offered under MPL-2.0. LibreNav is not affiliated with Google.
