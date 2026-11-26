# Contribuir a Aura

## Preparar el entorno

Los requisitos y la compilación están en el [README](README.md#compilar-desde-el-código). Lo
específico de Windows y macOS está en [docs/BUILD-PLATFORMS.md](docs/BUILD-PLATFORMS.md).

Arrancar la app con recarga en caliente:

```bash
cargo tauri dev
```

Es `cargo tauri`, no `pnpm tauri`: la CLI es la de Rust (`cargo install tauri-cli`).

## Formato

```bash
cargo fmt --all
```

`rustfmt.toml` está en el repo, así que `cargo fmt` respeta el estilo y no debería cambiar nada
fuera del código que tocaste. Deja el formateo fuera de los commits de funciones.

La interfaz no tiene configuración de Prettier: desactiva el formateo al guardar en `ui/` y sigue
el estilo del archivo en el que estás.

## Tests

```bash
cargo test --all                                        # todo, sin red
cargo test -p aura-app --lib -- --ignored --nocapture   # consulta las APIs de letras reales
cd ui && pnpm check && pnpm test                        # svelte-check, tipos y chequeos propios
bash scripts/check-aura-brand.sh all                    # sin rastros de la marca anterior
```

Los tests que usan la red llevan `#[ignore]` para que la ejecución normal funcione sin conexión.
Si tocas un proveedor de letras, corre los ignorados: un proveedor cuyo endpoint cambió devuelve
"sin letras" en vez de un error.

Las comprobaciones de extracción de YouTube en vivo van detrás de una feature:

```bash
cargo test -p innertube --features integration-tests -- --nocapture
```

Córrelas después de cambiar la lista de clientes, el cifrado o el camino de resolución.

En macOS los binarios de test enlazan libmpv igual que la app, así que necesitan el mismo
`LIBRARY_PATH`:

```bash
export LIBRARY_PATH="$(brew --prefix)/lib:$LIBRARY_PATH"
```

## Pull requests

- Una rama por cambio, con PR contra `main`.
- Un cambio por PR, con el porqué y lo que probaste.

## Traducciones

Las traducciones están en `ui/src/lib/locales/` como JSON anidado, un archivo por idioma, con
`en.json` como referencia.

- Los marcadores como `{count}` y `{playlist}` se sustituyen al ejecutar: escríbelos igual que en
  inglés; puedes moverlos dentro de la frase.
- Una clave que falta no es un error: cae al inglés.

Para añadir un idioma: crea el JSON, impórtalo en `ui/src/lib/locales/index.ts` y añádelo a
`LocaleId`, `LOCALES` y `translations`.

## Convenciones

- **Íconos: solo [HugeIcons](https://hugeicons.com)** (`@hugeicons/svelte` y
  `@hugeicons/core-free-icons`).
- **Componentes base: shadcn-svelte** antes de hacer uno a mano.
- **La interfaz nunca habla con YouTube.** Todo lo de YouTube queda detrás de los comandos de Tauri.
- **Los colores salen de los tokens del tema** (`--foreground`, `--muted-foreground`, …), nunca de
  hex o rgb fijos.
- **Los efectos visuales siguen [docs/UI-PERFORMANCE.md](docs/UI-PERFORMANCE.md).** Léelo antes de
  añadir un blur, sombra, animación al pasar el mouse o `backdrop-filter`.

## Alcance

Aura usa la API privada de YouTube y se rompe cuando YouTube cambia algo. Todo lo relacionado con la
extracción tiene que poder actualizarse: versiones de cliente en configuración y no fijas en el
código, y degradación ordenada cuando un paso falla.
