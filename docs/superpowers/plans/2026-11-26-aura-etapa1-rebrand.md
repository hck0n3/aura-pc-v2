# Aura — Etapa 1: renombrar a Aura y quitar lo del autor anterior — Plan de implementación

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Convertir el árbol de Limusic en "Aura" de hck0n3: sin Discord, sin letras comunitarias ni cuentas, sin Escuchar juntos ni enlace de dispositivos, sin infraestructura del autor anterior, con marca, ícono, autoría y canal de actualizaciones propios, y con todo lo demás de Limusic intacto.

**Architecture:** Borrado quirúrgico por funcionalidad (backend Rust en `src-tauri/`, UI SvelteKit en `ui/`, traducciones, CI), cada uno detrás de un guardián automático (`scripts/check-aura-brand.sh`) que falla mientras quede rastro. Luego renombrado de marca y repunte de URLs. Cada tarea deja el árbol compilando y con los tests de Limusic en verde.

**Tech Stack:** Rust (workspace, Tauri 2), SvelteKit + Svelte 5 en `ui/` (pnpm), libmpv, GitHub Actions.

**Spec:** `docs/superpowers/specs/2026-11-26-aura-hybrid-design.md` (§4, §10, §11, §11b)

## Global Constraints

- Licencia GPL-3.0-or-later. **No borrar ni editar** `LICENSE`, `THIRD-PARTY-NOTICES.md` ni líneas de copyright existentes.
- Nombre visible: **Aura**. Identificador: **`com.hck0n3.aura`**. Autor/publisher: **`hck0n3`**.
- Línea legal en Acerca de (texto exacto): `Aura © 2026 hck0n3. Basado en Limusic — GPL-3.0` con enlace a la licencia.
- Repo y releases: `https://github.com/hck0n3/aura-pc-v2`. Rama por defecto: `main`.
- Si algo ya lo tiene Limusic, se queda como está: no reescribir funciones que no estén en esta lista de borrado.
- Lecciones del dueño: no asumir firmas (grep la definición real), buscar TODOS los caminos de una función antes de borrarla, "compila" ≠ "funciona".
- Comandos de verificación (de `.github/workflows/checks.yml`): `cargo fmt --all --check`, `cargo test --all`, `cd ui && pnpm install --frozen-lockfile && pnpm check && pnpm test`.
- Dependencias de sistema Linux para compilar: `libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev libssl-dev libdbus-1-dev libmpv-dev`.

## Review Focus

1. **Ajustes guardados de funciones borradas** (filas `discord_rpc_config`, `limusic_token`, `listen_together*`, `device_link` en la base de settings): Aura no debe fallar al arrancar si existen; se ignoran. → test en Tarea 4.
2. **Proveedor de letras "Limusic" en el orden guardado por el usuario** (`lyrics_providers`): un orden guardado que lo mencione no debe romper la búsqueda de letras; el id desconocido se descarta y el resto sigue en su orden. → test en Tarea 3.
3. **Enlaces de invitación / deep links** (`ui/src/lib/invite.ts`, esquema de URL) que abrían salas de Escuchar juntos: abrir uno viejo no debe colgar ni mostrar pantalla rota; se ignora. → test en Tarea 4.
4. **Rutas que la UI navegaba y ya no existen** (`/review`, `/lyrics-editor`, `/team`): ningún botón, menú, atajo ni la paleta de comandos debe seguir llevando ahí. → paso de verificación en Tareas 3 y 5.
5. **Traducciones a medias**: quitar claves en `en.json` y no en las otras 40 hace que `pnpm test` (chequeo de locales) falle o muestre claves crudas; toda clave borrada se borra en todos los locales. → cubierto por el guardián y `pnpm test` en cada tarea.

---

## Mapa de archivos

| Funcionalidad | Se borra | Se edita |
|---|---|---|
| Discord | `src-tauri/src/discord.rs`, `ui/src/lib/discord.ts`, `ui/src/lib/components/Discord{Settings,Card,Icon}.svelte` | `src-tauri/src/lib.rs` (~l.12, ~1110), `state.rs`, `commands.rs`, `media.rs`, `src-tauri/Cargo.toml` (`discord-rich-presence`), `Cargo.lock`, `ui/src/lib/api.ts`, `player.svelte.ts`, `SettingsDialog.svelte`, `Sidebar.svelte`, `Titlebar.svelte`, `AppearanceSettings.svelte`, `ScrobbleSettings.svelte`, `settings-search.ts`, `lastfm.*`, `scrobble.ts`, locales |
| Letras comunitarias + cuentas | `src-tauri/src/limusic.rs`, `ui/src/lib/limusic{.ts,.svelte.ts,.check.ts}`, `ui/src/lib/review{.ts,.check.ts}`, `ui/src/routes/review/`, `ui/src/routes/lyrics-editor/` (su única salida es enviar a la comunidad), `LimusicAccountSettings.svelte`, componentes solo usados por el editor (p. ej. `LyricTimeline.svelte`, `lyricsedit.*` — confirmar con grep que nadie más los importa) | `src-tauri/src/lyrics.rs` (proveedor `LIMUSIC_URL`, `send_hint`), `commands.rs` (~l.2805–2910 `limusic_*`), `lib.rs` (registro de comandos), `+layout.svelte` (`watchLimusic`), `LyricsSourcePicker.svelte`, `lyricsSources.ts`, locales |
| Escuchar juntos + enlace de dispositivos | `src-tauri/src/listentogether/`, `src-tauri/src/devicelink.rs`, `crates/listen-protocol/`, `crates/sync-server/`, `ui/src/lib/lt.svelte.ts`, `ui/src/lib/link.svelte.ts`, `ui/src/lib/invite.ts` (si solo sirve a esto), componentes LT/Link | `Cargo.toml` (workspace members), `src-tauri/Cargo.toml`, `lib.rs` (~l.10, 24, 1131–1133 y puentes), `state.rs` (usos de `LtSession`, `LinkSession`, `shareable`, `publish_status`), `commands.rs`, `tauri.conf.json` (deep-link si es solo LT), locales |
| Infra del autor | `website/`, `ui/src/routes/team/`, `CommunityCard.svelte`, `.github/FUNDING.yml`, `.github/ISSUE_TEMPLATE/config.yml`, `.github/workflows/{website,stream-health}.yml`, `scripts/announce.sh`, `docs/*.md` del autor (todo `docs/` menos `docs/superpowers/` y las plantillas `*-template.css` que use el código) | `CONTRIBUTING.md`, `README.md`, `LanguagePicker.svelte` / `langlist.ts` / `i18n.svelte.ts` (enlaces a Weblate) |
| Marca | `src-tauri/icons/*` (regenerados) | `src-tauri/tauri.conf.json` (+ `.windows`/`.macos`), `src-tauri/Cargo.toml`, `ui/package.json`, `ui/src/app.html`, locales, `README.md`, cualquier texto visible con "Limusic" |
| Actualizaciones / release | — | `tauri.conf.json` (`updater`), `commands.rs` (~l.3040, ~3097), `ui/src/lib/updater.svelte.ts`, `.github/workflows/*.yml`, `scripts/release.sh` |
| Guardián | crear `scripts/check-aura-brand.sh` | `.github/workflows/checks.yml` |

---

### Task 0: Entorno y línea base

**Files:** ninguno.

- [ ] **Step 1:** Instalar dependencias de sistema (Global Constraints) con `apt-get`.
- [ ] **Step 2:** Desde `/home/user/aura-pc-v2` en rama nueva `aura/etapa1` (desde `aura/design`): `cargo test --all` y `cd ui && pnpm install --frozen-lockfile && pnpm check && pnpm test`.
  Expected: todo verde. Anotar en el PR cualquier test que ya fallara antes de tocar nada (línea base).

---

### Task 1: Guardián de marca

**Files:**
- Create: `scripts/check-aura-brand.sh`
- Modify: `.github/workflows/checks.yml` (añadir job; cambiar disparadores `branches: [master, dev]` → `[main]`)

**Interfaces:**
- Produces: `scripts/check-aura-brand.sh [discord|community|listen|infra|brand|all]` — sale 0 si no hay coincidencias de esa categoría, 1 y lista `archivo:línea` si las hay. Las tareas 2–6 lo usan como su test que falla primero.

- [ ] **Step 1: Escribir el script.** Busca con `git grep -nI -i` sobre archivos versionados, excluyendo siempre `LICENSE`, `THIRD-PARTY-NOTICES.md`, `docs/superpowers/`, `Cargo.lock`, `ui/pnpm-lock.yaml` y el propio script. Patrones por categoría:
  - `discord`: `discord`
  - `community`: `lyrics\.limusic\.app`, `limusic_token`, `LIMUSIC_URL`, `limusic_(providers|status|sign_in|sign_out|me|rename|vote|report|review)`
  - `listen`: `sync\.limusic\.app`, `listentogether`, `listen[-_]protocol`, `sync-server`, `devicelink`, `LtSession`, `LinkSession`
  - `infra`: `SimoHypers`, `ko-fi`, `weblate`, `limusic\.app`
  - `brand`: `limusic` (todo lo que quede), con una lista de permitidos línea a línea: la línea legal exacta `Basado en Limusic — GPL-3.0` y líneas `Copyright`.
  - `all`: todas.
- [ ] **Step 2:** `bash scripts/check-aura-brand.sh all` → Expected: FALLA listando cientos de coincidencias (ese es el estado inicial).
- [ ] **Step 3:** Añadir al job `ui` (o uno nuevo) de `checks.yml` el paso `bash scripts/check-aura-brand.sh all`; cambiar los `branches` de `checks.yml` y `karaoke-gpu-linux.yml` a `[main]`.
- [ ] **Step 4: Commit** `ci: guardián de marca de Aura`.

---

### Task 2: Quitar Discord

**Files:** fila "Discord" del mapa.

- [ ] **Step 1:** `bash scripts/check-aura-brand.sh discord` → FALLA (estado de partida).
- [ ] **Step 2:** Backend: borrar `discord.rs`, su `mod`, el `discord::spawn` de `lib.rs` y todo campo/canal/comando que lo alimente en `state.rs`, `commands.rs`, `media.rs`; quitar `discord-rich-presence` de `src-tauri/Cargo.toml` y `cargo update -p discord-rich-presence` para limpiar el lock. Buscar TODOS los usos con `grep -rn -i discord src-tauri crates` antes de dar por terminado.
- [ ] **Step 3:** UI: borrar `discord.ts` y los tres componentes; quitar sus imports/secciones en `SettingsDialog`, `Sidebar`, `Titlebar`, `AppearanceSettings`, `ScrobbleSettings`, `settings-search.ts`, `player.svelte.ts`, `api.ts`, `lastfm.svelte.ts`, `scrobble.ts`.
- [ ] **Step 4:** Locales: borrar de los 41 `ui/src/lib/locales/*.json` toda clave cuyo nombre o valor mencione Discord (script con `jq` o Node; misma lista de claves en todos los archivos).
- [ ] **Step 5:** `bash scripts/check-aura-brand.sh discord` → PASS (0 coincidencias). `cargo fmt --all --check && cargo test --all` → PASS. `cd ui && pnpm check && pnpm test` → PASS.
- [ ] **Step 6: Commit** `feat!: quitar integración con Discord`.

---

### Task 3: Quitar letras comunitarias, cuentas, revisión y editor de letras

**Files:** fila "Letras comunitarias" del mapa. Test: el módulo de tests existente de `src-tauri/src/lyrics.rs`.

**Interfaces:**
- Consumes: guardián `community`.
- Produces: `lyrics.rs` con el conjunto de proveedores `Boidu, Unison, LyricsPlus, LRCLIB, YouTube Music` (en ese orden por defecto) y la función existente que lee `lyrics_providers` (`provider_order`) ignorando ids desconocidos.

- [ ] **Step 1: Test que falla** en `lyrics.rs`:
```rust
#[test]
fn saved_order_with_removed_limusic_provider_drops_it_and_keeps_the_rest() {
    // orden guardado por un usuario de Limusic, con el proveedor ya inexistente en medio
    let order = provider_order(Some(r#"["lrclib","limusic","boidu"]"#));
    assert_eq!(order.first().map(|p| p.id()), Some("lrclib"));
    assert!(order.iter().all(|p| p.id() != "limusic"));
    assert_eq!(order.iter().position(|p| p.id() == "boidu"), Some(1));
}
```
  Ajustar nombres al tipo real (grep `fn provider_order` y el enum/struct de proveedor antes de escribirlo; no asumir la firma). Correr `cargo test -p limusic-app saved_order_with_removed` → FAIL (hoy "limusic" es válido).
- [ ] **Step 2:** Borrar el proveedor Limusic de `lyrics.rs` (`LIMUSIC_URL`, `LIMUSIC_TIMEOUT`, `send_hint`, su entrada en `PROVIDERS` y la documentación de cabecera) y `limusic.rs` completo; quitar los comandos `limusic_*` de `commands.rs` y su registro en `lib.rs`.
- [ ] **Step 3:** UI: borrar `limusic*.ts`, `review*.ts`, rutas `review/` y `lyrics-editor/`, `LimusicAccountSettings.svelte`, y los componentes/módulos que solo usaba el editor (confirmar cada uno con `grep -rn "<Nombre>" ui/src`). Quitar `watchLimusic` de `+layout.svelte`, la opción "Limusic" de `LyricsSourcePicker.svelte`/`lyricsSources.ts`, y cada botón/menú/atajo/entrada de `CommandPalette.svelte` que navegue a `/review` o `/lyrics-editor` (`grep -rn "lyrics-editor\|/review" ui/src`).
- [ ] **Step 4:** Locales: borrar claves `lyrics_editor.*`, `review.*`, cuentas Limusic y textos del proveedor en los 41 archivos.
- [ ] **Step 5:** Verificar: test del Step 1 PASS; `check-aura-brand.sh community` PASS; `cargo fmt --all --check && cargo test --all`, `pnpm check && pnpm test` PASS; `grep -rn "lyrics-editor\|'/review'" ui/src` sin resultados.
- [ ] **Step 6: Commit** `feat!: quitar letras comunitarias, cuentas y editor de letras`.

---

### Task 4: Quitar Escuchar juntos y enlace de dispositivos

**Files:** fila "Escuchar juntos" del mapa. Test: módulo de tests de `src-tauri/src/state.rs` o `db.rs` (donde se lean settings al arrancar) y un `*.check.ts` en `ui/src/lib/`.

- [ ] **Step 1: Tests que fallan.**
  - Rust: `fn startup_ignores_settings_rows_of_removed_features()` — una base de settings con filas `discord_rpc_config`, `limusic_token`, `listen_together_server`, `device_link` (grep los nombres reales de esas filas) se abre y el estado arranca sin error; ninguna de esas claves se vuelve a leer (`grep` lo confirma en Step 5).
  - UI: en `ui/src/lib/` el chequeo de deep links (o el que maneje `limusic://`/URL de invitación; grep el esquema en `tauri.conf.json` y `+layout.svelte`): un enlace de invitación viejo de sala se ignora sin lanzar excepción.
  Correr y ver FAIL (o no compilar) por las referencias que aún existen.
- [ ] **Step 2:** Backend: borrar `listentogether/`, `devicelink.rs`, sus `mod`, la creación de `LtSession`/`LinkSession` en `lib.rs` y los puentes de `LinkCommand`/`SyncCommand`; en `state.rs` quitar campos y ramas que los usan. `shareable()` y `publish_status` solo existen para LT/Link: borrar con sus llamadas, revisando cada uso (≈l.1405, 5572, 5847, 5949, 5971) para no romper la cola normal. Quitar `crates/listen-protocol` y `crates/sync-server` de `[workspace] members` y borrar las carpetas; quitar la dependencia en `src-tauri/Cargo.toml`.
- [ ] **Step 3:** UI: borrar `lt.svelte.ts`, `link.svelte.ts`, `invite.ts` si solo sirve a esto, y sus componentes/botones; deep-link de sala en `tauri.conf.json` solo si no lo usa nada más.
- [ ] **Step 4:** Locales: borrar claves de Escuchar juntos y de enlace de dispositivo en los 41 archivos.
- [ ] **Step 5:** Verificar: tests del Step 1 PASS; `check-aura-brand.sh listen` PASS; suites completas PASS; `grep -rn "listen_together\|device_link" src-tauri/src` solo dentro del test del Step 1.
- [ ] **Step 6: Commit** `feat!: quitar Escuchar juntos y enlace de dispositivos`.

---

### Task 5: Quitar infraestructura del autor anterior

**Files:** fila "Infra del autor" del mapa.

- [ ] **Step 1:** `check-aura-brand.sh infra` → FAIL.
- [ ] **Step 2:** Borrar `website/`, `ui/src/routes/team/`, `CommunityCard.svelte`, `.github/FUNDING.yml`, `.github/ISSUE_TEMPLATE/config.yml`, `.github/workflows/website.yml`, `.github/workflows/stream-health.yml`, `scripts/announce.sh`. De `docs/` dejar solo `docs/superpowers/` y las plantillas que el código cargue (`grep -rn "matugen-template\|pywal-template\|wallust-template" src-tauri ui/src` decide cuáles quedan).
- [ ] **Step 3:** Quitar enlaces a Weblate en `LanguagePicker.svelte`, `langlist.ts`, `i18n.svelte.ts` (las traducciones se quedan); quitar enlaces a `/team`, Ko-fi y Discord de cualquier menú/Acerca de.
- [ ] **Step 4:** Reescribir `README.md` (en español, corto): qué es Aura, plataformas, cómo compilar (los mismos pasos técnicos de antes), licencia GPL-3.0 y la línea "Basado en Limusic". Reescribir `CONTRIBUTING.md` sin enlaces ni nombres del autor, manteniendo las reglas técnicas (`rustfmt`, tests).
- [ ] **Step 5:** `check-aura-brand.sh infra` PASS; `pnpm check && pnpm test`, `cargo test --all` PASS; `grep -rn "/team" ui/src` sin resultados.
- [ ] **Step 6: Commit** `chore: quitar web, financiación e infraestructura del autor anterior`.

---

### Task 6: Marca Aura, ícono y autoría

**Files:** fila "Marca" del mapa. Fuente del ícono: `/home/user/jr-music-player-pc/assets/icon.png` (1024×1024 RGBA, del dueño).

- [ ] **Step 1:** `check-aura-brand.sh brand` → FAIL.
- [ ] **Step 2:** `tauri.conf.json`: `productName` `"Aura"`, `identifier` `"com.hck0n3.aura"`, título de ventana `"Aura"`, `publisher`/copyright del bundle `"hck0n3"`; mismo cambio en `tauri.windows.conf.json`/`tauri.macos.conf.json` donde repitan nombre. El test existente `macos_window_config_mirrors_the_base_window` debe seguir pasando.
- [ ] **Step 3:** `src-tauri/Cargo.toml`: `name = "aura-app"`, `description = "Aura — reproductor de escritorio para YouTube Music"`, `authors = ["hck0n3"]`; `[workspace.package]` en `Cargo.toml` con `authors = ["hck0n3"]`. Actualizar cualquier `-p limusic-app` en workflows/scripts.
- [ ] **Step 4:** Ícono: copiar el PNG del dueño a `src-tauri/icons/source.png` y `cd ui && pnpm tauri icon ../src-tauri/icons/source.png -o ../src-tauri/icons`; también reemplazar cualquier logo de Limusic en `ui/static/` y `assets/` (grep nombres de archivo con "limusic").
- [ ] **Step 5:** Textos: en los 41 locales reemplazar "Limusic" por "Aura" en valores (no en claves); renombrar archivos/identificadores de código que contengan `limusic` (p. ej. claves de `localStorage` `limusic:*` → `aura:*`, constantes, variables de entorno `LIMUSIC_*` → `AURA_*`), con grep tras cada renombre.
- [ ] **Step 6:** Acerca de (`SettingsDialog.svelte` o el componente que muestre la versión): mostrar exactamente `Aura © 2026 hck0n3. Basado en Limusic — GPL-3.0` con enlace a `https://www.gnu.org/licenses/gpl-3.0.html`. Añadir la clave `about.legal` a `en.json` y `es.json` (el resto cae a inglés).
- [ ] **Step 7:** `check-aura-brand.sh all` PASS (solo queda la línea legal permitida); suites completas PASS; `cd ui && pnpm build` PASS.
- [ ] **Step 8: Commit** `feat!: Aura de hck0n3 — nombre, ícono, identificador y autoría`.

---

### Task 7: Actualizaciones y releases desde `hck0n3/aura-pc-v2`

**Files:** fila "Actualizaciones / release" del mapa.

- [ ] **Step 1:** `tauri.conf.json` `plugins.updater.endpoints` → `["https://github.com/hck0n3/aura-pc-v2/releases/latest/download/latest.json"]`; **borrar la `pubkey` del autor anterior** y poner `bundle.createUpdaterArtifacts: false` hasta que el dueño genere su clave (instrucciones en el PR: `pnpm tauri signer generate`, privada en el secret `TAURI_SIGNING_PRIVATE_KEY`, pública en `pubkey`). Mientras no haya clave, el chequeo de actualizaciones en la UI (`updater.svelte.ts`) no debe mostrar error: si no hay `pubkey`, no consulta.
- [ ] **Step 2:** `commands.rs` (~l.3040 API de releases, ~l.3097 `latest.json` beta): owner/repo → `hck0n3/aura-pc-v2`. Grep `releases/` en `src-tauri ui/src` para no dejar ninguno.
- [ ] **Step 3:** Workflows de release y `scripts/release.sh`: repo `hck0n3/aura-pc-v2`, artefactos con nombre `Aura_*`; la descarga fijada de `libmpv-2.dll` pasa a `https://github.com/hck0n3/aura-pc-v2/releases/download/deps/<mismo nombre de archivo>` con el mismo hash SHA-256 que ya verifica el workflow (el dueño sube el archivo al release `deps`; dejar en el PR el enlace exacto de origen para que lo baje y lo suba).
- [ ] **Step 4:** `check-aura-brand.sh all` PASS; suites completas PASS.
- [ ] **Step 5: Commit** `ci: releases y actualizaciones desde hck0n3/aura-pc-v2`.

---

### Task 8: Comprobación real y entrega

- [ ] **Step 1:** `cd ui && pnpm tauri build --debug` (Linux) → binario generado. Arrancarlo con `xvfb-run` y comprobar en el log que no hay errores de arranque; captura de pantalla de la ventana principal y de Ajustes → Acerca de (nombre Aura, ícono nuevo, línea legal, sin Discord ni cuentas).
- [ ] **Step 2:** Push de `aura/etapa1` y PR en borrador contra `main` con: lo que se quitó, lo que el dueño debe hacer (subir `libmpv-2.dll` al release `deps`; generar la clave de actualizaciones), y qué no se pudo probar (Windows).
- [ ] **Step 3:** Disparar `windows-release.yml` en modo `dry_run` cuando el DLL esté subido, para que el dueño pruebe el instalador en su PC.
