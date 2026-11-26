# Aura — Etapa 2: el DSP de JR en la cadena de audio de mpv — Plan de implementación

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Que el DSP de JR Music Player PC (EQ de 10 bandas "Aura" con sus presets, EQ paramétrico de 24 bandas, loudness, bass, exciter, tube, compresor multibanda, ancho estéreo, crossfeed / espacial, diálogo, de-esser, limitador) y la elección del dispositivo de salida suenen en Aura con **todo** lo que se reproduce (YouTube, local, descargas, radio), configurable en una pestaña nueva de Ajustes y guardado entre sesiones.

**Architecture:** Un módulo puro en `crates/player` traduce `DspSettings` a un grafo lavfi de ffmpeg; el reproductor lo inserta como un eslabón etiquetado `@dsp` en la cadena `af` de mpv que ya lleva ganancia, reverb y tempo, así que viaja solo a la segunda cubierta del crossfade y sobrevive al gapless. El backend guarda los ajustes como JSON en la tabla de settings y los aplica al arrancar; la UI es una pestaña "Audio" en Svelte con los presets y perfiles de JR.

**Tech Stack:** Rust (crate `player`, libmpv2), Tauri 2, SvelteKit/Svelte 5 en `ui/`, filtros lavfi de ffmpeg (verificados en el `libmpv-2.dll` del release `deps`: equalizer, lowshelf, highshelf, highpass, lowpass, acompressor, acrossover, alimiter, crossfeed, aexciter, asoftclip, extrastereo, deesser, asplit, amix, pan, adelay, aecho, volume).

**Spec:** `docs/superpowers/specs/2026-11-26-aura-hybrid-design.md` §6 (enfoque A aprobado)

**Fuente del comportamiento (solo lectura):** repo `hck0n3/jr-music-player-pc`: `native/src/jr_audio_engine.cpp` (motor), `src/ui/renderer/js/native_dsp_ui.js` (UI y presets de 24 bandas, líneas 55-239), `src/ui/renderer/js/aura_eq.js` (10 bandas y sus 18 presets, líneas 36-73; auto-headroom 158-166), `src/ui/renderer/dsp_pro.html` (etiquetas). Se toma **el comportamiento del motor C++**: JR tenía dos implementaciones distintas (C++ para local, Web Audio para YouTube) y Aura tiene una sola para todo.

## Global Constraints

- **El DSP viene apagado por defecto.** Con `enabled: false` la cadena `af` es byte a byte la de antes de esta etapa: gapless, normalización, tempo, tono, reverb y karaoke de Limusic no cambian.
- Un solo camino: lo que suena por YouTube y por local pasa por el mismo `@dsp`. Nada en Web Audio.
- Sin Superpowered ni código cerrado. GPL-3.0.
- Orden de la cadena `af`: ganancia (normalización) → `@dsp` → `@reverb` → estiramiento (tempo/tono).
- Orden dentro de `@dsp` (el del callback de JR, `jr_audio_engine.cpp:793-1047`): preamp → HPF 20 Hz → EQ (10 o 24 bandas) → de-esser → loudness → espacial **o** crossfeed → bass → exciter → tube → compresor multibanda → ancho → diálogo → limitador.
- Rangos (de la UI de JR): preamp −20..+6 dB; ganancia de banda ±15 dB; Q 0.1..10; frecuencia 20..20000 Hz; cantidades 0..1; ancho 0..2; espacial 0..3.
- Cambios en vivo: se reconstruye la cadena con `af set` (como ya hace Limusic, que nunca usa `af-command`). La UI no manda más de una actualización cada 120 ms mientras se arrastra un slider.
- Toda clave de ajuste nueva va en `UI_SETTINGS` (`src-tauri/src/commands.rs`, el tipo lleva la longitud).
- Textos de UI en `en.json` y `es.json`; el resto de idiomas cae a inglés. Etiquetas de JR en español donde ya existan (`dsp_pro.html`).
- Comandos de verificación: `cargo fmt --all --check`, `cargo test --all` (salvo el test de crossfade en tiempo real conocido por intermitente), `cd ui && pnpm check && pnpm test`, `bash scripts/check-aura-brand.sh all`.
- Rama `aura/etapa2-dsp` desde `main`; PR contra `main`.

## Review Focus

1. **Todo encendido a la vez** (cada efecto al máximo, 24 bandas a +15 dB, preamp +6): el grafo debe ser válido para ffmpeg y la salida no puede pasar del techo del limitador ni dar NaN (el bug de JR con `shelfSlope`=Q en los shelves). → tests en Tareas 1 y 2.
2. **Cambio de canción con el DSP encendido**, en gapless y con crossfade: la canción siguiente suena con el mismo DSP (la normalización reconstruye la cadena en cada avance; la cubierta B la hereda). → test en Tarea 2.
3. **Ajustes guardados corruptos o de una versión anterior** (JSON roto, campo desconocido, valor fuera de rango): Aura arranca con el DSP por defecto, sin error. → test en Tarea 3.
4. **Un efecto que ffmpeg rechace** (filtro ausente en el libmpv de alguna plataforma): la reproducción sigue con la cadena anterior y la UI muestra el error, en vez de quedarse muda. → test en Tarea 2.
5. **Dispositivo de salida guardado que ya no existe** (auriculares USB desconectados): suena por el dispositivo por defecto. → test en Tarea 4.

---

## Mapa de archivos

| Archivo | Responsabilidad |
|---|---|
| `crates/player/src/dsp.rs` (nuevo) | Tipos `DspSettings` y compañía, valores por defecto, validación, `dsp_graph()` |
| `crates/player/src/lib.rs` | `AudioFilters.dsp`, `@dsp` en `apply_filters`, `set_dsp`/`dsp`, dispositivos de salida, `INHERITED` |
| `ui/src/lib/dsp-defaults.json` (nuevo) | Los valores por defecto, una sola fuente para Rust (test) y TS (import) |
| `src-tauri/src/commands.rs`, `state.rs`, `lib.rs` | Comandos `get_dsp`/`set_dsp`/`audio_devices`, guardado, aplicación al arrancar |
| `ui/src/lib/dsp.ts` + `dsp.check.ts` (nuevos) | Tipos TS, presets de JR, auto-headroom, parser AutoEQ, validación de perfiles |
| `ui/src/lib/components/AudioSettings.svelte` (nuevo) | La pestaña "Audio" |
| `ui/src/lib/components/SettingsDialog.svelte`, `settings-search.ts`, `api.ts`, `player.svelte.ts`, locales | Enganche de la pestaña, API y textos |

---

### Task 1: `DspSettings` y el grafo lavfi (puro)

**Files:**
- Create: `crates/player/src/dsp.rs` (+ `pub mod dsp;` y reexport en `crates/player/src/lib.rs`)
- Create: `ui/src/lib/dsp-defaults.json`
- Test: módulo `#[cfg(test)]` en `dsp.rs`

**Interfaces:**
- Produces (todas `#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]`, `#[serde(rename_all = "camelCase", default)]`):
  - `pub struct DspSettings { enabled: bool, preamp_db: f64, eq: Eq, deesser: bool, loudness: bool, spatial: u8, crossfeed: bool, bass: Amount, exciter: Amount, tube: Amount, mb_comp: bool, width: Amount, dialogue: Amount, limiter: bool }`
  - `pub struct Amount { on: bool, amount: f64 }`
  - `pub struct Eq { mode: EqMode, aura: [f64; 10], bands: Vec<EqBand> }`
  - `pub enum EqMode { Off, Aura, Parametric }` (`#[serde(rename_all = "lowercase")]`)
  - `pub struct EqBand { f: f64, g: f64, q: f64, kind: BandKind }`, `pub enum BandKind { Peak, LowShelf, HighShelf }` (lowercase)
  - `impl Default for DspSettings` con los valores de JR: `enabled false`, `preamp_db 0`, `eq.mode Aura`, `aura` todo 0, `bands` = rejilla de 24 (20, 31, 50, 80, 100, 160, 200, 315, 440, 500, 800, 1000, 1600, 2000, 2500, 3150, 4000, 5000, 6300, 8000, 10000, 12500, 16000, 20000 Hz; g 0; q 4.318; kind Peak), `deesser false`, `loudness false`, `spatial 0`, `crossfeed false`, `bass {false, 0.28}`, `exciter {false, 0.15}`, `tube {false, 0.25}`, `mb_comp false`, `width {false, 1.0}`, `dialogue {false, 0.35}`, `limiter true`.
  - `pub fn validate(&self) -> Result<(), String>`: rangos de Global Constraints; `bands.len() == 24`; nombre del campo en el error.
  - `pub fn dsp_graph(d: &DspSettings) -> Option<String>`: `None` si `!d.enabled`; si no, el grafo lavfi (sin envolver en `lavfi=`).
  - `pub const AURA_FREQS: [f64; 10] = [31.5, 62.5, 125.0, 250.0, 500.0, 1000.0, 2000.0, 4000.0, 8000.0, 16000.0];`

- [ ] **Step 1: Tests que fallan** (en `dsp.rs`):
  - `default_is_off_and_has_no_graph`: `dsp_graph(&DspSettings::default()) == None`.
  - `defaults_match_the_ui_copy`: `serde_json::to_value(DspSettings::default())` == el contenido de `ui/src/lib/dsp-defaults.json` (`include_str!("../../../ui/src/lib/dsp-defaults.json")`).
  - `enabled_but_flat_is_only_preamp_hpf_and_limiter`: con `enabled: true` y todo lo demás por defecto, el grafo contiene `highpass=f=20` y `alimiter` y **no** contiene `equalizer`, `acompressor`, `crossfeed`, `asplit`.
  - `flat_bands_are_skipped`: una banda con `|g| <= 0.01` no aparece; una de +3 dB a 1000 Hz aparece como `equalizer=f=1000:t=q:w=4.318:g=3`.
  - `aura_bands_use_shelves_at_the_ends`: modo Aura con `aura[0]=4.5`, `aura[9]=2` → `lowshelf=f=31.5` y `highshelf=f=16000` con pendiente `t=s:w=0.6`; una banda intermedia es `equalizer=…:t=q:w=1.414`.
  - `shelf_slope_is_capped`: en modo Parametric, un shelf con q 4.318 y g +15 sale con `t=s:w=1` (nunca mayor que 1: es el NaN de JR).
  - `spatial_wins_over_crossfeed`: `spatial: 2, crossfeed: true` → el grafo tiene la etapa espacial y no `crossfeed=`.
  - `validate_rejects_out_of_range`: `preamp_db: 7.0` → `Err` que menciona `preampDb`; `bands` con 23 elementos → `Err`.
- [ ] **Step 2:** `cargo test -p player dsp::` → FAIL (no compila: módulo ausente).
- [ ] **Step 3:** Implementar `dsp.rs`. **Cómo se arma el grafo:** cada etapa es un texto con un pad de entrada `[in]` y uno de salida `[out]`. El ensamblador las encadena reemplazando `[out]` de una y `[in]` de la siguiente por la misma etiqueta única (`s0`, `s1`, …), y quita el `[in]` de la primera y el `[out]` de la última, porque mpv conecta los pads sin nombre. Las etapas lineales son `[in]filtro[out]`. Etapas (valores del motor C++ de JR):
  - preamp: `volume={preamp_db}dB` (se omite si es 0).
  - HPF: `highpass=f=20:t=q:w=0.707`.
  - EQ Aura: por banda con `|g| > 0.01`; la 0 `lowshelf`, la 9 `highshelf` (`t=s:w=0.6`), el resto `equalizer` `t=q:w=1.414`. Auto-headroom de JR: `volume` de `-max(0, maxBoost - 1.5)` dB delante del EQ cuando haga falta.
  - EQ paramétrico: por banda con `|g| > 0.01`: Peak → `equalizer`; shelves → `lowshelf`/`highshelf` con `t=s:w=min(q, 1)`.
  - de-esser: `deesser=i=0.4:m=0.5:f=0.27` (6.5 kHz a 48 kHz).
  - loudness: `lowshelf=f=200:t=s:w=0.707:g=6,highshelf=f=5000:t=s:w=0.707:g=3.5` (el punto medio del rango adaptativo de JR).
  - espacial (perfiles 1–3 de `jr_audio_engine.cpp:1373-1391`, D/S/R e ITD): `[in]asplit[d][c];[c]pan=stereo|c0=c1|c1=c0,adelay=delays={itd_muestras}S|{itd_muestras}S,lowpass=f=700,volume={S}[x];[d]volume={D}[dd];[dd][x]amix=inputs=2:normalize=0,aecho=1:1:12|19:{R}|{R}[out]`, con ITD en muestras a 48 kHz.
  - crossfeed: `crossfeed=strength=0.35:range=0.5`.
  - bass: `bass=g={amount*10}:f=100:w=0.5`.
  - exciter: `aexciter=amount={amount*10}:freq=6000`.
  - tube (fórmula de JR, `cpp:930-942`): drive = 1+6a, wet = min(1, 1.5a), dry = 1−wet: `[in]asplit[d][w];[w]volume={drive},asoftclip=type=tanh,volume={wet/drive}[t];[d]volume={dry}[dd];[dd][t]amix=inputs=2:normalize=0[out]`.
  - compresor multibanda (`cpp:341-350`): `[in]acrossover=split=200 5000[lo][mi][hi];[lo]acompressor=threshold=0.10:ratio=2.5:attack=10:release=150[a];[mi]acompressor=threshold=0.16:ratio=1.8:attack=10:release=150[b];[hi]acompressor=threshold=0.10:ratio=3:attack=10:release=150[c];[a][b][c]amix=inputs=3:normalize=0,volume=1.15[out]`.
  - ancho: `extrastereo=m={amount}`.
  - diálogo: `[in]asplit[o][v];[v]pan=mono|c0=0.5*c0+0.5*c1,highpass=f=300,lowpass=f=3000,volume={amount},pan=stereo|c0=c0|c1=c0[vv];[o][vv]amix=inputs=2:normalize=0[out]`.
  - limitador: `alimiter=limit=0.891:attack=5:release=100:level=disabled:latency=1` (−1 dBFS; `latency=1` como el de Limusic, por el gapless).

  Los números se formatean sin ceros de más (`3`, no `3.000000`). Escribir `ui/src/lib/dsp-defaults.json` con el `Default` serializado (camelCase).
- [ ] **Step 4:** `cargo test -p player dsp::` → PASS.
- [ ] **Step 5: Commit** `feat(dsp): ajustes del DSP de JR y su grafo lavfi`.

---

### Task 2: `@dsp` en la cadena de mpv

**Files:**
- Modify: `crates/player/src/lib.rs` (`AudioFilters` ~:122, `apply_filters` ~:935, nuevo `set_dsp`/`dsp` junto a `set_playback_params` ~:841)
- Test: módulo de tests de `lib.rs` (reusar los arneses `ao=pcm` de `reverb_renders_a_tail_and_updates_both_decks` ~:2024 y `transition_tests.rs` `render` ~:60)

**Interfaces:**
- Consumes: `dsp::{DspSettings, dsp_graph}` (Task 1).
- Produces: `pub fn set_dsp(&self, dsp: DspSettings) -> Result<(), Error>` (valida → `Error::InvalidParams`; aplica a las dos cubiertas; si `af set` falla, vuelve a aplicar la cadena anterior en ambas y devuelve el error, igual que `set_playback_params`); `pub fn dsp(&self) -> DspSettings`.

- [ ] **Step 1: Tests que fallan:**
  - `dsp_off_leaves_the_chain_as_before`: con el DSP por defecto, la cadena que lee `af` de un mpv real (patrón de `mpv_keeps_the_gain_through_pitch_changes_and_failures`) no contiene `@dsp` y es igual a la de antes de esta tarea para ganancia −3 dB, tono +2 y reverb.
  - `dsp_sits_between_gain_and_reverb`: con DSP encendido, reverb y tempo: `af` contiene, en este orden, `volume=`, `@dsp:lavfi=graph=`, `@reverb:`.
  - `everything_on_is_a_valid_graph`: `DspSettings` con todo encendido, cantidades al máximo, ancho 2, espacial 3, EQ paramétrico con las 24 bandas a +15 y shelves en los extremos, preamp +6: `validate_lavfi` del eslabón `@dsp` → `Ok`.
  - `eq_boost_is_heard`: render `ao=pcm` de ruido rosa (`anoisesrc=color=pink`) con EQ paramétrico +12 dB a 1000 Hz frente al mismo render sin DSP: la energía de la banda 900–1100 Hz (`band_energy` de `transition_tests.rs`) sube al menos 8 dB.
  - `limiter_holds_the_ceiling`: ruido a 0 dBFS con preamp +6 y bass y tube al máximo: pico de salida ≤ −0.9 dBFS.
  - `zero_width_is_mono`: ancho 0 → en la salida L y R difieren en menos de −60 dB.
  - `dsp_survives_a_track_change`: dos pistas en cola (gapless) con DSP encendido; tras el avance (que reconstruye la cadena por la ganancia nueva), `af` sigue conteniendo `@dsp`; y `preload` a la cubierta B copia un `af` con `@dsp`.
  - `a_rejected_dsp_keeps_playing_the_old_chain`: con la costura de test que ya existe para filtros ausentes (`NO_RUBBERBAND` ~:1142, o una equivalente `NO_DSP_FILTER` que cambie un nombre de filtro del grafo por uno inexistente), `set_dsp` devuelve `Err` y `af` sigue siendo la cadena anterior en ambas cubiertas.
- [ ] **Step 2:** `cargo test -p player` → los nuevos FAIL.
- [ ] **Step 3:** Implementar. `AudioFilters` gana `dsp: DspSettings`. En `apply_filters`, tras la ganancia: si `dsp_graph` da `Some(g)`, `@dsp:lavfi=graph=%{len}%{g}` (longitud en bytes, como `@reverb`). `set_filter_chain` sondea también cuando la cadena trae `@dsp` y la cubierta está fría. `set_dsp` copia el patrón de rollback de `set_playback_params`.
- [ ] **Step 4:** `cargo test -p player` → PASS (salvo el crossfade en tiempo real conocido).
- [ ] **Step 5: Commit** `feat(dsp): el DSP suena en la cadena de mpv, en las dos cubiertas`.

---

### Task 3: Backend: comandos, guardado y arranque

**Files:**
- Modify: `src-tauri/src/state.rs` (junto a `set_playback_params` ~:3447), `src-tauri/src/commands.rs`, `src-tauri/src/lib.rs` (registro de comandos ~:1397; arranque ~:1067-1073 junto a `saved_crossfade`)
- Test: módulo de tests de `state.rs` (o `commands.rs`, donde viva `saved_crossfade`)

**Interfaces:**
- Consumes: `Player::set_dsp`, `Player::dsp` (Task 2).
- Produces:
  - Comandos Tauri `get_dsp() -> player::DspSettings` y `set_dsp(dsp: player::DspSettings) -> Result<(), String>`. `set_dsp` aplica, guarda el JSON en el setting `dsp` y emite `dsp-changed` con los ajustes aplicados.
  - `fn saved_dsp(db: &Db) -> DspSettings`: lee el setting `dsp`; JSON roto, inválido (`validate`) o ausente → `DspSettings::default()`, con un `tracing::warn!`.
  - Setting `dsp_profiles` (JSON de perfiles con nombre, lo escribe la UI con `setSetting`) en `UI_SETTINGS`.

- [ ] **Step 1: Tests que fallan:** `saved_dsp_falls_back_to_default_on_bad_json` (valores `"{"`, `"{\"preampDb\": 99}"`, `"{\"unknownField\": 1, \"enabled\": true}"` → el tercero se acepta con `enabled: true`, los dos primeros dan el default) y `saved_dsp_round_trips` (guardar unos ajustes y leerlos da lo mismo).
- [ ] **Step 2:** `cargo test -p aura-app --lib saved_dsp` → FAIL.
- [ ] **Step 3:** Implementar. Al arrancar, `player.set_dsp(saved_dsp(&db))` antes del bucle de eventos (si falla, se registra y se sigue sin DSP). `set_dsp` toma el mismo cerrojo `playback_effects` que `set_playback_params`.
- [ ] **Step 4:** `cargo test -p aura-app --lib` → PASS.
- [ ] **Step 5: Commit** `feat(dsp): guardar y restaurar el DSP; comandos get/set`.

---

### Task 4: Dispositivo de salida

**Files:**
- Modify: `crates/player/src/lib.rs` (`INHERITED` ~:1372, nuevas funciones), `src-tauri/src/commands.rs`, `src-tauri/src/lib.rs`
- Test: tests de `crates/player` con `ao=null`

**Interfaces:**
- Produces: `pub struct AudioDevice { pub name: String, pub description: String }` (Serialize); `pub fn audio_devices(&self) -> Vec<AudioDevice>` (de la propiedad `audio-device-list`, siempre con `auto` primero); `pub fn set_audio_device(&self, name: &str) -> Result<(), Error>` (las dos cubiertas). Comando `audio_devices`. Setting `audio_device` (en `UI_SETTINGS`), con efecto en `set_setting` y aplicado al arrancar; vacío = `auto`.

- [ ] **Step 1: Tests que fallan:** `device_list_starts_with_auto`; `an_unknown_saved_device_falls_back_to_auto` (con un nombre que no existe la cubierta sigue reproduciendo y `audio-device` queda en `auto`); `the_idle_deck_inherits_the_device` (`INHERITED` incluye `audio-device`).
- [ ] **Step 2:** `cargo test -p player audio_device` → FAIL.
- [ ] **Step 3:** Implementar. Si `set_audio_device` con un nombre guardado falla al arrancar, se aplica `auto` y se registra.
- [ ] **Step 4:** PASS.
- [ ] **Step 5: Commit** `feat(audio): elegir el dispositivo de salida`.

---

### Task 5: `dsp.ts`: tipos, presets y perfiles (UI, puro)

**Files:**
- Create: `ui/src/lib/dsp.ts`, `ui/src/lib/dsp.check.ts`
- Modify: `ui/src/lib/api.ts` (`getDsp`, `setDsp`, `onDspChanged`, `audioDevices`)

**Interfaces:**
- Consumes: `dsp-defaults.json` (Task 1), comandos de las Tareas 3 y 4.
- Produces:
  - Tipos TS iguales a los de Rust (`DspSettings`, `Amount`, `Eq`, `EqMode`, `EqBand`, `BandKind`); `DSP_DEFAULTS` (import del JSON).
  - `AURA_PRESETS: { id: string; name: string; gains: number[] }[]`: los 18 de `aura_eq.js:36-73`, en ese orden, con "Audiophile" como el del botón Restablecer.
  - `PARAMETRIC_PRESETS`: los de `native_dsp_ui.js:55-239` (Dispositivo, Efecto, Género), con sus bandas y los efectos que encienden (Auriculares → crossfeed; Dolby Atmos/DTS → espacial 1/2/3; Loudness y Noche con su preamp).
  - `SPATIAL_PROFILES`: 0 Bypass (Estéreo puro), 1 Íntimo (cerca), 2 Amplio (equilibrado), 3 Cine (envolvente).
  - `parseAutoEq(text: string): EqBand[] | null`: líneas `Filter N: ON PK|LSC|HSC Fc <hz> Hz Gain <db> dB Q <q>` → hasta 24 bandas ordenadas por frecuencia, rellenando con bandas planas de la rejilla; `null` si no hay ninguna línea válida.
  - `readProfiles(raw: string | null): DspProfile[]` (`DspProfile = { name: string; dsp: DspSettings }`): lanza si el JSON está roto, para que un guardado no pise lo que hay (mismo criterio que `readEffectPresets`).

- [ ] **Step 1: `dsp.check.ts` que falla:** `AURA_PRESETS.length === 18` y `Harman` = `[4.5, 3.5, 1, -0.5, 0, 0, 1.5, 2.5, 1, 0.5]`; todos los presets paramétricos tienen 24 bandas dentro de rango; `parseAutoEq('Filter 1: ON PK Fc 105 Hz Gain -3.2 dB Q 0.70')` da una banda Peak a 105 Hz con −3.2/0.7 y el resto plano; `parseAutoEq('hola')` → `null`; `readProfiles('{')` lanza; `readProfiles(null)` → `[]`.
- [ ] **Step 2:** `cd ui && node --experimental-strip-types src/lib/dsp.check.ts` → FAIL.
- [ ] **Step 3:** Implementar copiando los valores de los archivos de JR citados.
- [ ] **Step 4:** `pnpm test` → PASS.
- [ ] **Step 5: Commit** `feat(dsp): presets y perfiles de JR en la UI`.

---

### Task 6: La pestaña "Audio" de Ajustes

**Files:**
- Create: `ui/src/lib/components/AudioSettings.svelte`
- Modify: `SettingsDialog.svelte` (pestaña `audio` tras `playback`, ícono de HugeIcons tipo ecualizador), `settings-search.ts` (`SettingsTab` + filas), `player.svelte.ts` (estado `dsp` alimentado por `dsp-changed`), `locales/en.json` y `es.json`

**Interfaces:**
- Consumes: Tarea 5.

- [ ] **Step 1: Tests que fallan:** filas de búsqueda para cada control (`settings-search.check.ts` falla mientras la pestaña tenga `row({ title: t('…') })` sin entrada); en `dsp.check.ts`, una función pura `debounce120` (o la que use la pestaña) que, con 10 llamadas en 50 ms, emite 1 durante el arrastre y la última al soltar.
- [ ] **Step 2:** `pnpm test` → FAIL.
- [ ] **Step 3:** La pestaña, con los componentes de `ui/src/lib/components/ui/*` y el patrón `section/row` de `SettingsDialog`:
  - **General:** "DSP activo" (interruptor maestro), Pre-amp (−20..+6, paso 0.5), Dispositivo de salida (select de `audioDevices`).
  - **Ecualizador:** modo (Apagado / Aura 10 bandas / Paramétrico 24 bandas). Aura: select de los 18 presets y 10 sliders verticales ±15. Paramétrico: select de presets por grupos, 24 columnas con slider ±15, Hz, Q y P/LS/HS, y "Importar AutoEQ (.txt)".
  - **Procesado:** Loudness; Bass perceptual, Ancho estéreo (0..2; al encenderlo en 1.0 salta a 1.5, como JR), Exciter armónico, Calidez de válvula, Realce de diálogo (cada uno interruptor + slider); Compresor multibanda; Audio espacial (select de `SPATIAL_PROFILES`) y Crossfeed, excluyentes.
  - **Masterización:** Limitador, De-esser.
  - **Perfiles:** guardar con nombre, cargar, borrar (setting `dsp_profiles`).
  - Cada cambio → `api.setDsp` con el envío limitado a uno cada 120 ms; un error del backend → toast y la UI vuelve a lo que confirmó `dsp-changed`.
- [ ] **Step 4:** `pnpm check && pnpm test` → PASS; `pnpm build` OK.
- [ ] **Step 5: Commit** `feat(dsp): pestaña Audio en Ajustes`.

---

### Task 7: Comprobación real y entrega

- [ ] **Step 1:** `cargo build -p aura-app --features tauri/custom-protocol`, abrir con Xvfb, capturas de la pestaña Audio (modo Aura y modo Paramétrico).
- [ ] **Step 2:** Render de comparación (con el arnés de la Tarea 2 o `decode_to_wav` con `af`): un clip de prueba sin DSP, con preset Harman y con espacial "Cine", y el espectro de cada uno, para el PR.
- [ ] **Step 3:** README: añadir "DSP: ecualizador Aura de 10 bandas y paramétrico de 24, loudness, bass, exciter, válvula, compresor multibanda, ancho, espacial, diálogo y limitador, en todo lo que suena".
- [ ] **Step 4:** Revisión final de toda la rama, push y PR contra `main`. Anotar en el PR lo que **no** se portó y por qué:
  - dither y orden de SRC: mpv entrega float y remuestrea con swresample, y JR solo los aplicaba en local;
  - analizador de espectro;
  - exportar/importar perfiles a archivo JSON (los perfiles se guardan en Aura);
  - el modo "puro" de Superpowered: el EQ Aura de 10 bandas ocupa su lugar.
