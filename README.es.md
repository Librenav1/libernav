# LibreNav

[Esperanto](README.md) · [Français](README.fr.md) · [English](README.en.md) · [Español](README.es.md)

LibreNav es un prototipo libre de navegador de escritorio: una interfaz familiar inspirada en Chromium, impulsada por Servo en lugar de Chromium. No copia código cerrado, marcas ni servicios de Google.

## Inicio rápido

```sh
./scripts/run-librenav.sh
```

Para la primera compilación en Fedora, instala las dependencias del [README principal](README.md) y ejecuta `./mach build --dev --media-stack dummy`.

## Funciones actuales

- Varias pestañas y ventanas
- Omnibox para direcciones y búsquedas
- Navegación, recarga y página de inicio
- Marcadores persistentes almacenados localmente
- Interfaz oscura y accesible según las convenciones de Chromium
- Página de pestaña nueva local en esperanto, francés, inglés y español
- Sin CDN, fuentes remotas ni telemetría añadida

## Límites

LibreNav todavía no sustituye a un navegador maduro. Las extensiones, la sincronización, la impresión y varias funciones avanzadas quedan por implementar. La compatibilidad depende del estado actual de Servo.

## Validación

```sh
./mach fmt
./mach check --media-stack dummy -p servoshell
```

El código de Servo conserva MPL-2.0 y sus licencias de terceros. Los cambios de LibreNav también se ofrecen bajo MPL-2.0. LibreNav no está afiliado a Google.
