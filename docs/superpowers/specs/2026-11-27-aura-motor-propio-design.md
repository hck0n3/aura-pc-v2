# Aura — Motor de audio propio: diseño y plan por fases

Fecha: 2026-11-27 · Dueño: hck0n3 · Reemplaza el enfoque A del §6 de
`2026-11-26-aura-hybrid-design.md` (DSP como filtros lavfi de mpv).

## 1. Por qué

La etapa 2 hizo el DSP de JR con filtros de ffmpeg dentro de la cadena `af` de mpv. Al probarlo en
Windows, el dueño encontró que **empeora el sonido**. Las causas están medidas y son de arquitectura:

| Síntoma | Causa | Prueba |
|---|---|---|
| Truena al mover una barra | Cada cambio reconstruye toda la cadena `af`; el estado de los filtros se reinicia | `moving_an_eq_band_while_playing_does_not_click`: salto 5.9× |
| No se puede ajustar en vivo | `af-command` no aplica cambios con este libmpv (3 sintaxis probadas) | `experiment_af_command_on_a_named_band` |
| Subir una banda baja el volumen | Auto-headroom de JR, dinámico | código |
| Los efectos distorsionan | Normalización a −7 LUFS + limitador, efectos no lineales a 48 kHz sin sobremuestreo (aliasing), constantes del C++ más agresivas que el camino de YouTube de JR | análisis + revisión |

Un DSP de calidad necesita cambiar parámetros **suavemente y sin reconstruir nada**, sobremuestrear
lo no lineal y controlar la salida. Eso no se puede hacer dentro de mpv.

## 2. Qué es el motor propio

```
 YouTube (Opus/AAC) ─┐                         ┌──────────── Aura (Rust) ────────────┐
 Local (FLAC, MP3…) ─┼─► mpv: demux + decodifica ─► PCM float ─► DSP de JR ─► volumen ─► salida
                     │   (+ tempo, tono, reverb)    (tubería)   (portado)   + limitador  WASAPI /
 Video ──────────────┘   mpv sigue mostrando video                                    CoreAudio /
                                                                                      ALSA-Pulse
```

- **mpv sigue haciendo** lo que hace bien: red, YouTube, demux, decodificación (con **libopus**), gapless,
  tempo/tono/reverb, video. Su salida de audio pasa a ser `ao=pcm` hacia una tubería
  (named pipe en Windows, FIFO en Linux/macOS) en float 32 bits.
- **Aura hace**: el DSP de JR portado a Rust casi línea por línea, el volumen, el limitador final, la
  mezcla del crossfade y la salida al dispositivo (crate `cpal`: WASAPI compartido o **exclusivo**,
  CoreAudio, ALSA/Pulse).
- **Todo en coma flotante de 64 bits** dentro del DSP. Parámetros **suavizados** (rampas de 10–30 ms) para
  que nada truene. Efectos no lineales con **sobremuestreo 4×**.

## 3. Objetivos / no-objetivos / anti-objetivos

**Objetivos**
- Mover cualquier control sin truenos ni cortes (medido).
- Cada efecto suena **igual que el motor C++ de JR** (comparado muestra a muestra con el código
  original compilado como referencia), con sobremuestreo donde JR no lo tenía.
- Mismo sonido en YouTube y local. Bit-perfect posible con modo exclusivo y DSP apagado.
- Nada de lo que Limusic ya hace se pierde: gapless, crossfade, normalización, tempo/tono/reverb,
  karaoke, video, letras sincronizadas, controles del sistema.

**No-objetivos**
- Reescribir la decodificación o el streaming (siguen en mpv).
- Superpowered (comercial, incompatible con GPL).

**Anti-objetivos**
- Que el DSP apagado suene distinto al audio original (debe ser transparente: test nulo).
- Que las letras o el video se desincronicen.
- Que un fallo del motor deje a Aura muda: siempre existe la salida clásica de mpv como respaldo.

## 4. Enfoques considerados

- **A. Filtros lavfi en mpv** (hecho en etapa 2): falla por las razones del §1.
- **B. mpv decodifica → tubería → motor propio** (ELEGIDO): control total del sonido sin rehacer
  streaming ni decodificación.
- **C. Reemplazar mpv por decodificación propia** (ffmpeg/symphonia): habría que rehacer streaming,
  gapless, video y tempo. Demasiado grande y arriesgado.

## 5. Fases

Cada fase termina con algo que se puede medir y, desde la Fase 2, con un instalador que el dueño
prueba. Cada fase tiene su propio plan detallado (`docs/superpowers/plans/`) antes de empezar.

### Fase 0 — Prueba de concepto (spike, desechable)
**Objetivo:** confirmar que la tubería funciona en Windows antes de invertir en lo demás.
- mpv escribe PCM float por un named pipe; un programa mínimo en Rust lo lee y lo reproduce con `cpal`.
- Medir: latencia de punta a punta, CPU, comportamiento al pausar, buscar (seek) y cambiar de pista.
- Correr en GitHub Actions (Windows) y en un ejecutable de prueba que el dueño abre en su PC.
- **Decisión go/no-go**: si la tubería no es viable en Windows, se replantea antes de seguir.

### Fase 1 — El motor de salida (sin DSP todavía)
**Objetivo:** Aura reproduce por su propio motor, sonando idéntico al original.
- Crate `crates/engine`: lectura de la tubería, búfer circular, salida `cpal`.
- Modo compartido y **modo exclusivo WASAPI** (la tarjeta cambia a la frecuencia de cada canción).
- Remuestreo de alta calidad (`rubato`, ya usado por el karaoke) solo cuando la tarjeta lo exige.
- Volumen dentro del motor (misma curva perceptual que hoy), con rampa.
- **Reloj y posición**: la posición que ve la app = la de mpv menos la latencia del motor, para que
  letras, barra de progreso y video sigan en sincronía.
- Pausa, búsqueda (vaciar el búfer al instante), pérdida y cambio de dispositivo.
- Respaldo: un ajuste oculto vuelve a la salida clásica de mpv.
- **Tests:** test nulo (salida == entrada, bit a bit con volumen 100), latencia medida, búsqueda sin
  audio viejo, dispositivo desconectado.

### Fase 2 — Integración con todo lo de Limusic
**Objetivo:** nada se rompe con el motor nuevo.
- Gapless (mpv sigue entregando continuo), **crossfade mezclado en el motor** (dos cubiertas → dos
  tuberías → mezcla de potencia constante), normalización a **−14 LUFS con 6 dB de margen fijo**,
  tempo/tono/reverb (siguen en mpv, antes del motor), karaoke, video musical (compensar retardo),
  controles multimedia del sistema.
- **Tests:** los de Limusic en verde, más gapless sin hueco, crossfade sin salto de volumen,
  sincronía de letras ±20 ms.
- **Entrega:** primer instalador con el motor nuevo (DSP aún desactivado) para que el dueño
  confirme que suena igual o mejor que antes.

### Fase 3 — El DSP de JR, portado
**Objetivo:** el DSP suena como el de JR, pero sin truenos ni aspereza.
- Portar la cadena del callback de `jr_audio_engine.cpp` (orden de §6 del diseño original): HPF,
  EQ 24 bandas (biquads RBJ como miniaudio), EQ Aura 10 bandas (con la pendiente de shelf correcta),
  de-esser, loudness adaptativa, espacial HRTF **con su sala de verdad** (realimentación filtrada),
  crossfeed, bass, exciter, válvula, multibanda, ancho, diálogo, limitador.
- **Sobremuestreo 4×** en bass, exciter y válvula (sin aliasing).
- **Rampas** en todos los parámetros; cambiar un preset hace un fundido corto entre configuraciones.
- Limitador de **picos reales con anticipación** (true-peak, lookahead), techo −1 dBFS.
- Dither TPDF solo si la salida es de 16 bits.
- **Referencia de paridad:** el motor C++ de JR compilado con `HAS_SUPERPOWERED=0` como
  "oráculo" en los tests; cada efecto debe coincidir con él (antes de añadir sobremuestreo, y
  dentro de una tolerancia medida después).
- **Tests por efecto:** paridad con el C++, mover cada control durante la reproducción sin salto,
  todo al máximo sin pasar del techo ni NaN, DSP apagado = transparente.

### Fase 4 — Ajustes y pantalla
**Objetivo:** controlarlo todo desde la pestaña Audio.
- Se reutiliza la pestaña de la etapa 2 (presets, perfiles, AutoEQ, dispositivo).
- Nuevos: modo exclusivo, frecuencia de salida (automática o fija), panel "Señal" (formato de
  entrada, frecuencia, bits, lo que hace el motor), preamp con margen fijo visible.
- Se quita el auto-headroom dinámico.

### Fase 5 — Validación con el dueño y entrega
- Instalador de Windows; lista de escucha guiada (cada efecto, YouTube y local, A/B contra JR).
- Rendimiento: CPU del motor < 3 % en una PC normal; 8 horas reproduciendo sin cortes ni deriva.
- Ajustes finos según lo que el dueño escuche. PR y unión a `main`.

## 6. Riesgos

| Riesgo | Mitigación |
|---|---|
| Named pipe de mpv inestable en Windows | Fase 0 lo prueba primero; alternativa: hilo propio con `ao=pcm` a archivo en RAM (circular) |
| Desincronía de letras/video por la latencia del motor | Reloj compensado en Fase 1, tests de ±20 ms |
| Bluetooth / dispositivos con latencia alta | Latencia leída del dispositivo y sumada al reloj |
| CPU con sobremuestreo 4× | Solo en las 3 etapas no lineales; medido en Fase 5 |
| Algo falla en el motor | Respaldo a la salida clásica de mpv con un ajuste |

## 7. Mientras tanto

La etapa 2 está en `main` con el DSP apagado por defecto: quien no lo active no nota nada. Hasta la
Fase 3, la recomendación es dejar el DSP apagado.
