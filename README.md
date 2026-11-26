# Aura

**Reproductor de escritorio para YouTube Music. Rust + Tauri, sin anuncios, sin Electron.**

Aura habla directamente con la API interna de YouTube y reproduce el audio con libmpv: sin un
navegador empaquetado, sin servidor propio y sin anuncios en el audio. Funciona en Windows, Linux y
macOS.

Aura © 2026 hck0n3. Basado en Limusic — GPL-3.0

---

## Funciones

**Escuchar**

- Sin anuncios, directo de la API de YouTube
- Sin cortes entre canciones (gapless), con normalización de volumen
- Radio y automix para que la cola no se acabe
- Videos musicales, radio en vivo y podcasts
- Tempo, tono y reverb, con presets
- También tus archivos locales

**Letras**

- Sincronizadas, palabra por palabra cuando la fuente lo tiene
- Dúos, coros y traducciones
- Ocho fuentes, en el orden que elijas
- Karaoke: quita la voz en tu propia computadora

**Tu biblioteca**

- Inicia sesión, con varias cuentas a la vez
- Playlists, me gusta, álbumes y artistas guardados, subidas
- Crear, editar y reordenar playlists
- Historial con los mismos días que YouTube Music
- Estadísticas de escucha, calculadas en tu equipo
- Búsqueda con vista previa mientras escribes
- Scrobbling a Last.fm y ListenBrainz

**En tu escritorio**

- Mini reproductor y modo teatro a pantalla completa
- Ícono en la bandeja, arranque opcional al iniciar sesión
- Teclas multimedia y "reproduciendo ahora" en cada sistema
- `Ctrl+K` para buscar, `Ctrl+H` para ver todos los atajos
- Se actualiza solo

**A tu gusto**

- Paletas de acento, colores y fuentes propios
- Colores que se adaptan a la portada que suena
- Material You, o los colores de tu fondo de pantalla con Matugen, wallust o pywal
- Ícono de la app personalizable
- Veintiséis idiomas

---

## Compilar desde el código

Fedora:

```bash
sudo dnf install mpv-libs mpv-libs-devel webkit2gtk4.1-devel \
  gcc gcc-c++ make openssl-devel librsvg2-devel
cd ui && pnpm install && cd ..
cargo tauri build
```

Ubuntu / Debian:

```bash
sudo apt-get install libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev \
  librsvg2-dev libssl-dev libdbus-1-dev libmpv-dev
cd ui && pnpm install && cd ..
cargo tauri build
```

Las instrucciones de Windows y macOS están en [docs/BUILD-PLATFORMS.md](docs/BUILD-PLATFORMS.md).

---

## Cómo funciona, en breve

- Un crate de Rust habla la API InnerTube de YouTube, haciéndose pasar por varios clientes
  oficiales y cambiando de uno a otro cuando alguno falla.
- Las URLs de los streams de YouTube están protegidas por JavaScript ofuscado (el cifrado de la
  firma y el parámetro `n`) y por la verificación de BotGuard. Aura ejecuta ese JavaScript donde
  espera ejecutarse, en un webview real y oculto, y nunca deja que toque la interfaz.
- El audio pasa por libmpv: transiciones sin cortes, caché en disco y normalización de volumen con
  los datos del propio YouTube.
- La interfaz es una SPA de SvelteKit que solo habla con el núcleo en Rust. Nunca contacta a
  YouTube por su cuenta.

---

## Aviso

Este proyecto no está afiliado, financiado, autorizado ni respaldado por YouTube, Google LLC ni
ninguna de sus filiales.

Todas las marcas y derechos de propiedad intelectual mencionados pertenecen a sus dueños.

---

## Licencia

[GPL-3.0](LICENSE).

El karaoke usa los modelos UVR-MDX-NET-Inst_HQ_3, UVR-MDX-NET-Inst_Main y UVR_MDXNET_3_9662 de
[Ultimate Vocal Remover](https://github.com/Anjok07/ultimatevocalremovergui) de Anjok07 y aufr33
(MIT) y, en Windows, ONNX Runtime y DirectML de Microsoft. Ver
[THIRD-PARTY-NOTICES.md](THIRD-PARTY-NOTICES.md).
