# AGENTS.md

Guía para agentes de IA (y personas) que trabajen en **pomo**, un Pomodoro de terminal en Rust con `ratatui` + `crossterm`. Ver `README.md` para la descripción funcional y la tabla completa de atajos.

## Objetivo

CLI/TUI interactiva, controlada por teclado, que se comporta como un programa de terminal estándar: entra en pantalla alterna, sale con `Ctrl+C`/`q`/`Ctrl+D` y **siempre** deja la terminal restaurada.

## Comandos

```bash
cargo build
cargo run -- -w 25 -s 5 -l 15
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt --check
cargo run --features notify      # notificaciones de escritorio
```

Ejecutar `fmt`, `clippy` y `test` antes de dar una tarea por terminada.

## Arquitectura

Separación simple al estilo hexagonal: el dominio no conoce la terminal.

| Capa | Archivos | Reglas |
|---|---|---|
| Dominio | `src/domain/{phase,timer}.rs` | Lógica pura. Sin `crossterm`, `ratatui`, `std::io` ni `Instant::now()`; el tiempo entra por parámetro. 100 % testeable. |
| Aplicación | `src/app.rs`, `src/input.rs` | `App` guarda estado y aplica `Action`. `input` solo traduce `KeyEvent → Action`. |
| Presentación | `src/ui.rs` | Solo dibuja desde `&App`. No muta estado. |
| Infraestructura | `src/terminal.rs`, `src/config.rs`, `src/notify.rs`, `src/audio.rs`, `src/cli.rs` | I/O: terminal, archivos, señales, notificaciones, audio. |
| Entrada | `src/main.rs` | Ensambla todo y corre el bucle. Sin lógica de negocio. |

Flujo por iteración: `draw` → `event::poll(200ms)` → `map_key` → `app.handle(action)` → `app.on_tick(Instant::now())`.

## Reglas obligatorias

### Terminal
1. Toda entrada a modo raw/pantalla alterna debe tener su `restore()` correspondiente, también ante error o `panic` (panic hook instalado **antes** de `init()`).
2. `restore()` debe ser idempotente.
3. En modo raw `Ctrl+C` llega como `KeyEvent` (`Char('c')` + `CONTROL`), no como `SIGINT`. Siempre mapearlo a `Action::Quit`.
4. Filtrar `KeyEventKind::Press`; ignorar `Release`/`Repeat` (importante en Windows).
5. Registrar `SIGTERM`/`SIGHUP` con `signal-hook` (solo `cfg(unix)`) y revisar el flag en el bucle.
6. Manejar `Event::Resize` redibujando; nunca asumir un tamaño fijo.

### Tiempo
7. El temporizador se basa en `Instant` (monotónico), restando `now - last_tick`. **No** contar ticks ni usar `thread::sleep` para medir.
8. `tick` no avanza si el estado no es `Running`. Al pausar/reanudar se reinicia `last_tick` para no contar el tiempo en pausa.

### Código
9. Sin `unwrap()`/`expect()` fuera de tests; usar `anyhow::Result` en infraestructura y `Option`/`Result` tipados en dominio.
10. Nada de `println!` mientras la TUI está activa (rompe el render). Mensajes al usuario van por `App.message` o a `stderr` tras `restore()`.
11. Los atajos se definen **solo** en `input.rs`. Si se agrega uno, actualizar también la ayuda en `ui.rs` y la tabla del `README.md`.
12. Los errores de notificaciones/campana/sonido se ignoran; nunca deben cerrar la app.
13. Configuración: valores por defecto razonables si falta el archivo; los flags de CLI tienen prioridad sobre el TOML. Duraciones mínimas de 1 minuto.

## Cómo agregar cosas

**Nuevo atajo**
1. Añadir variante a `Action` en `input.rs`.
2. Mapear la tecla en `map_key` + test.
3. Manejar la acción en `App::handle`.
4. Actualizar popup de ayuda (`ui.rs`) y tabla del README.

**Nueva fase o regla del ciclo**
1. Cambiar `Phase`/`Timer` en `domain/`.
2. Agregar tests de transición antes de tocar la UI.
3. Reflejarlo en `ui.rs` (color/etiqueta) y `config.rs` si es configurable.

**Nueva opción de configuración**
1. Campo en `Config` (con `serde(default)`), flag en `cli.rs`, lógica de merge.
2. Documentar en `README.md`.

## Pruebas esperadas

- `domain/timer.rs`: trabajo→descanso corto; 4.º pomodoro→descanso largo; descanso→trabajo; `skip`; `reset`; `adjust` con mínimo 1; `tick` en pausa no avanza; `tick` con `Instant` simulado.
- `input.rs`: un test por atajo, incluyendo `Ctrl+C` y `Ctrl+D`, y que `Release` se ignore.
- `config.rs`: defaults, merge con CLI, archivo inexistente.

La UI se verifica manualmente (`cargo run`) y, si se desea, con `ratatui::backend::TestBackend` para comprobar que el render no hace panic en tamaños pequeños (p. ej. 20×5).

## Checklist de revisión manual

- [ ] `Ctrl+C`, `q`, `Esc`, `Ctrl+D` salen y la terminal queda normal (cursor visible, sin caracteres raros).
- [ ] Un `panic!` forzado restaura la terminal.
- [ ] `kill -TERM <pid>` sale limpio.
- [ ] Redimensionar la ventana no rompe la UI; en tamaño muy pequeño no hay panic.
- [ ] Pausar 10 s y reanudar no descuenta ese tiempo.
- [ ] Al terminar una fase suena la campana / sonido envolvente y cambia a la siguiente.
- [ ] `?` abre/cierra la ayuda y lista todos los atajos reales.

## Fuera de alcance (por ahora)

Estadísticas persistentes, sincronización en red, tareas por pomodoro. Si se añaden, respetar la separación de capas y documentarlas aquí.