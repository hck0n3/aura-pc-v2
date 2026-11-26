# Aura — diseño del híbrido (Limusic + JR Music Player PC)

Fecha: 2026-11-26 · Dueño: hck0n3

## 1. Intención

El dueño quiere un reproductor de escritorio llamado **Aura** que tenga **todo lo que ya hace
Limusic** (cliente nativo de YouTube Music en Rust + Tauri 2 + Svelte, audio por libmpv) y además
tres cosas de su propio proyecto **JR Music Player PC / Aura Hi-Res** (Electron + C++ N-API):

1. **Su DSP**, que debe sonar **sí o sí también con el audio de YouTube** (en su proyecto así funciona).
2. **El Radar de Novedades** de artistas seguidos.
3. **La descarga de música en la máxima calidad posible.**

Además: renombrar a Aura, quitar todo lo de Discord, que la firma/autoría sea del dueño y que no
quede rastro del autor anterior en nombres, créditos o colaboraciones, **salvo lo que la GPL-3.0
obliga a conservar** (ver §4).

Regla del dueño: **si algo de lo que pidió ya lo tiene Limusic, se queda lo de Limusic** y no se
porta. Por eso la reproducción local NO se porta (Limusic ya la tiene: `src-tauri/src/local.rs`),
ni crossfade/gapless/automix, ni normalización de volumen, ni tempo/pitch/reverb.

## 2. Fuentes

| Repo | Rol | Ruta en sesión |
|---|---|---|
| `SimoHypers/limusic` (GPL-3.0, v1.5.0-rc.2, commit d9f7dcf) | Base | `/home/user/limusic` |
| `hck0n3/jr-music-player-pc` (versión más nueva, 2026-11-26) | Origen de DSP, Radar, descarga | `/home/user/jr-music-player-pc` |
| `hck0n3/aura-hires-pc` | Versión anterior del mismo programa; solo referencia | `/home/user/aura-hires-pc` |

JR PC y Aura Hi-Res PC son el **mismo código**; manda JR PC.

## 3. Objetivos / no-objetivos / anti-objetivos

**Objetivos**
- Aura arranca, se compila e instala en Windows (y sigue compilando en Linux/macOS como Limusic).
- El DSP se aplica a TODO lo que suena: YouTube, local, descargas, radio. Un solo camino de audio.
- Radar de Novedades con la misma lógica que JR PC.
- Descarga en máxima calidad sin recomprimir.
- Branding Aura completo, sin Discord, autoría del dueño.

**No-objetivos**
- Sonido bit-idéntico al motor C++ de JR PC (se aceptó el enfoque A: equivalentes de ffmpeg).
- Portar la UI Electron tal cual; se rehace en Svelte con el estilo de Limusic.
- Portar funciones de JR PC no pedidas (nube, licencias/activación, onboarding, video mode, etc.).

**Anti-objetivos (sería un fracaso aunque "funcione")**
- Que el DSP solo funcione con archivos locales.
- Romper gapless/automix/normalización/tempo/karaoke/video que Limusic ya tiene.
- Incluir Superpowered (SDK comercial cerrado; incompatible con distribuir GPL-3).
- Borrar avisos de copyright/licencia que la GPL exige (sería una violación de licencia).
- Que Aura siga llamando a servidores del autor anterior con su marca.

## 4. Marca, autoría y licencia

**Se renombra** Limusic → Aura en: `productName`, `identifier` (nuevo: `com.hck0n3.aura`),
ventana/título, íconos, textos de UI y las 40+ traducciones (`ui/src/lib/locales/*.json`), README,
nombres de módulos/archivos con "limusic" (`src-tauri/src/limusic.rs`, `ui/src/lib/limusic*.ts`, etc.)
y nombres de crates/paquetes donde aparezca. Autor/publisher en `Cargo.toml`, `package.json`,
instalador: el dueño (nombre exacto: ver §11).

**Se elimina por completo:**
- Discord: `src-tauri/src/discord.rs` (Rich Presence), `DiscordSettings/DiscordCard/DiscordIcon.svelte`,
  `ui/src/lib/discord.ts`, la dependencia en `src-tauri/Cargo.toml`, enlaces y claves de traducción
  relacionadas, y el login "con Discord" de cuentas.
- `website/`, `ui/src/routes/team/`, `.github/FUNDING.yml` (Ko-fi), `.github/ISSUE_TEMPLATE/config.yml`
  (enlaces del autor), `scripts/announce.sh`, `CommunityCard.svelte`, integración con Weblate,
  badges/enlaces del autor en README y CONTRIBUTING.
- Menciones de Metrolist como "origen" en textos de marketing (se conserva solo lo legal).

**Se conserva (obligatorio por GPL-3.0):** `LICENSE`, `THIRD-PARTY-NOTICES.md`, y cualquier línea
de copyright existente. Se añade el copyright del dueño para sus cambios y, en *Ajustes → Acerca de*,
una línea discreta: "Aura © 2026 <dueño>. Basado en Limusic — GPL-3.0" con enlace a la licencia
(GPL-3 §5a y §5d: aviso de modificación y "Appropriate Legal Notices"). El dueño ya fue informado y
aceptó este punto.

**Servidores del autor anterior** (decisión en §11): `lyrics.limusic.app` (letras comunitarias +
cuentas), `sync.limusic.app/ws` (Escuchar juntos), updater en `github.com/SimoHypers/limusic`.
El updater se apunta a `github.com/hck0n3/aura/releases` con una **clave de firma nueva** (la
privada la genera el dueño y va en GitHub Secrets; nunca al repo).

## 5. Enfoques considerados (elegido: A)

- **A. Traducir el DSP a filtros lavfi de libmpv/ffmpeg** — ELEGIDO. Suena en YouTube y local
  porque todo pasa por mpv; multiplataforma; sin Superpowered. Coste: no idéntico al C++.
- B. Portar el DSP C++ a Rust y sacar el audio de mpv para procesarlo. Idéntico, pero libmpv no
  expone hook de audio; obligaría a reemplazar la salida y arriesga gapless/automix. Semanas.
- C. Reemplazar libmpv por el motor miniaudio/WASAPI de JR. Rehacer casi todo el reproductor.

Acordado: si tras A algún efecto concreto no convence de sonido, se porta ese efecto a mano (B
puntual), como trabajo aparte.

## 6. DSP (enfoque A)

**Dónde se engancha.** `crates/player/src/lib.rs` ya mantiene una cadena `af` etiquetada
(`@gain`, `@reverb`, rubberband) y la retoca con `af-command`; también valida cadenas con
`validate_lavfi`. El DSP de Aura se añade como **un bloque etiquetado más** en esa misma cadena,
con orden fijo, sin tocar el de tempo/pitch/reverb/gain existentes.

**Orden propuesto** (del motor JR, `native/src/jr_audio_engine.cpp`):
preamp → EQ (10 ISO o paramétrico 24 bandas) → de-esser → loudness → bass enhance → exciter →
tube → multibanda → width → crossfeed **o** HRTF (mutuamente excluyentes) → dialogue → limitador.
El builder puede reordenar dentro de lo razonable si un filtro lo exige; el limitador va último.

**Mapeo de controles JR → filtro ffmpeg** (función N-API de JR entre paréntesis):

| Control JR | Filtro lavfi | Nota |
|---|---|---|
| Preamp (`setPreAmp`, `setSpPreamp`) | `volume` | |
| EQ 10 bandas ISO, Q=1.414, banda 0 low-shelf, 9 high-shelf, ±18 dB (`setSpEqBand`) | `equalizer`/`lowshelf`/`highshelf` por banda | 0 dB = banda fuera de la cadena (bypass real, como JR) |
| EQ paramétrico 16–24 bandas + tipo por banda (`setPEQNode`, `setBandType`) | biquads por banda (`equalizer`, `lowshelf`, `highshelf`, `lowpass`, `highpass`, `bandpass`, `notch`) | rejilla 1/3 octava + 440 Hz como JR |
| De-esser 6.5 kHz | `deesser` o `adynamicequalizer` | |
| Loudness (`setLoudness`) | shelves de contorno | aproximación |
| Bass enhance (`setBassEnhance`) | `bass` / `asubboost` | |
| Exciter (`setHarmonicExciter`) | `aexciter` | |
| Tube warmth (`setTubeWarmth`) | `asoftclip` (tanh/atan) + drive | aproximación declarada |
| Multibanda 3 bandas LR (`setMBComp`) | `acrossover` → `acompressor`×3 → `amix` | grafo lavfi |
| Stereo width (`setStereoWidth`) | `stereotools`/`extrastereo` | |
| Crossfeed (`setCrossfeed`) | `crossfeed` | |
| HRTF / perfiles espaciales (`setHRTF`, `setSpatialProfile`) | `sofalizer` + archivo SOFA con licencia libre empaquetado; si el build de mpv no trae libmysofa, `headphone`/crossfeed reforzado | verificar |
| Dialogue (`setDialogueEnhance`) | `dialoguenhance` | ffmpeg ≥ 6 |
| Limitador (`setLimiter`, `setSuperpoweredLimiter`) | `alimiter` | |
| Dither (`setDither`) | `aresample` dither_method | solo aplica si hay reducción de bits; declarar |
| Oversampling (`setOversampling`) | `aresample` arriba/abajo alrededor de etapas no lineales | opcional |
| On/Off global (`setDSPEnabled`) | quitar el bloque entero | bypass real |
| Dispositivo de salida (`getOutputDevices`, `setOutputDevice`) | propiedades mpv `audio-device-list` / `audio-device` | Limusic no lo tiene → se añade |

**Ya lo tiene Limusic, no se porta:** ReplayGain/normalización (`setReplayGain`), crossfade y
precarga (`preloadNext`, `startCrossfade`), espectro/visualizador si Limusic ya muestra uno.
`setSpPureMode` (cadena "pura" Superpowered) desaparece: el EQ de 10 bandas ES ese modo.

**Cambios en vivo.** Mover un slider no debe cortar el audio: usar `af-command` sobre filtros que
aceptan comandos (volume, equalizer, acompressor, crossfeed, alimiter…). Cambios estructurales
(añadir/quitar una etapa, activar HRTF) reconstruyen el bloque; es aceptable un microcorte, como ya
ocurre en Limusic al activar reverb.

**Verificación obligatoria.** Antes de comprometerse a cada filtro, comprobar que existe en el
mpv/ffmpeg que Limusic empaqueta en Windows (`validate_lavfi` / `mpv --af=help`). Si uno falta,
usar la alternativa de la tabla y anotarlo. "Compila" ≠ "suena": verificar el efecto con audio
real (p. ej. render con `ao=pcm` como en `crates/player/src/decode.rs` y medir).

**UI.** Limusic no tiene ecualizador ni pantalla de efectos (solo el diálogo de tempo/tono/reverb,
`TempoPitchDialog.svelte`, que se queda como está). Pantalla "DSP" nueva en Ajustes o en el reproductor, en Svelte con los componentes de Limusic
(`ui/src/lib/components/ui/*`), basada en la estructura de `src/ui/renderer/dsp_pro.html` y
`native_dsp_ui.js` de JR (secciones, rangos, presets). Presets propios guardados en la base de
datos de ajustes de Limusic (`src-tauri/src/db.rs`). Textos en español e inglés; otras
traducciones caen al inglés.

## 7. Radar de Novedades

Lógica de JR (`src/core/main.js`, `_getRadarTracks`, ~línea 1248) a portar en Rust usando
`crates/innertube`:
- Lista de artistas seguidos. Por cada uno: buscar artista → perfil → singles y álbumes del año
  actual y el anterior → el lanzamiento más nuevo → hasta 3 temas.
- Caché semanal con clave "viernes más reciente" + hash de la lista seguida; botón "Actualizar"
  que fuerza.
- Peticiones en prioridad baja y espaciadas (JR escanea hasta 80 artistas) para no disparar límites
  de YouTube ni competir con la reproducción.
- Vista "Radar de Novedades" en la barra lateral; reproducir todo / tema suelto con el reproductor
  de Limusic.

**Artistas seguidos (decisión del builder, propuesta):** botón "Seguir" en la página de artista,
guardado localmente en la base de datos de Limusic (funciona sin iniciar sesión). Si el usuario
inició sesión con YouTube, se suman también sus suscripciones (`get_library_subscriptions`).

## 8. Descarga en máxima calidad

JR PC usa yt-dlp + ffmpeg y en el modo rápido **recomprime a MP3 ~190 kbps**, lo que pierde calidad.
En Aura:
- Elegir el **mejor formato de audio** que ofrece YouTube para ese tema (normalmente Opus ~160 kbps
  itag 251; AAC 256 kbps itag 141 cuando la cuenta Premium lo da), con la misma resolución de
  streams que Limusic ya usa para reproducir (`audioproxy.rs`, `crates/innertube`, cipher/potoken).
- **Guardar el flujo original sin recomprimir** (remux a `.opus`/`.m4a`), con etiquetas (título,
  artista, álbum, año, número) y carátula incrustada.
- Lo descargado aparece en la biblioteca local de Limusic y suena con el DSP.
- Descargar tema, álbum y playlist; cola con progreso y cancelar; carpeta configurable.
- No se empaqueta yt-dlp salvo que la resolución propia de Limusic no alcance.

## 9. Restricciones

- Licencia GPL-3.0-or-later para todo Aura. Nada de Superpowered.
- Toolchain de Limusic (Rust workspace, Tauri 2, SvelteKit en `ui/`, libmpv). Seguir sus
  convenciones (`CONTRIBUTING.md`, `rustfmt.toml`).
- El dueño prueba en Windows; aquí se compila/prueba en Linux. Instalador Windows por GitHub
  Actions (ya existe `.github/workflows/windows-release.yml`, adaptar a Aura).
- El dueño debe poder empaquetar y correr Aura en un equipo limpio (regla permanente de su proyecto).
- No reintroducir errores documentados en `jr-music-player-pc/docs/audit/LECCIONES.md` y
  `docs/REGRESSION_REGISTRY.md` (p. ej. no asumir firmas; buscar TODOS los caminos).

## 10. Repositorio y entrega

- Repo nuevo `hck0n3/aura` (el dueño lo crea; esta sesión no puede crear repos) con la historia de
  Limusic como base, para conservar autoría de commits ajena tal como exige la GPL y poder traer
  arreglos futuros.
- Trabajo en ramas + PRs en borrador, por etapas: (1) rebranding y quitar Discord, (2) DSP,
  (3) Radar, (4) Descargas. Cada etapa compila y pasa los tests de Limusic.

## 11. Decisiones pendientes del dueño

(Se rellenan con las respuestas del dueño.)

## 11b. Decisiones tomadas por el diseñador (el dueño puede corregirlas)

- **Dependencias de build y actualizaciones** se mudan a releases de `hck0n3/aura`: el
  `libmpv-2.dll` fijado que hoy baja el workflow de Windows desde `SimoHypers/limusic/releases/
  download/deps/` (el dueño lo sube una vez a un release `deps` de su repo), el chequeo de versiones
  por API de GitHub (`commands.rs` ~3040) y el `latest.json` del canal beta (`commands.rs` ~3097).
  Se mantienen los canales estable y beta.
- **Sin migración de datos** en la primera versión: Aura es una app nueva (`com.hck0n3.aura`), no lee
  datos de Limusic ni de JR PC. Importar artistas seguidos/presets desde JR queda como mejora futura.
  Por eso los artistas seguidos se guardan por ID de canal de YouTube (más fiable que por nombre).


## 12. Queda a criterio del builder

Estructura interna de módulos Rust, nombres de comandos Tauri, diseño visual fino de las pantallas
nuevas dentro del estilo de Limusic, orden exacto de etapas DSP dentro de lo indicado, archivo SOFA
concreto (con licencia libre), formato de los presets.
