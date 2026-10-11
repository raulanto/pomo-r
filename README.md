# pomo — Pomodoro de terminal en Rust

Aplicación TUI (interfaz de terminal) para trabajar con la técnica Pomodoro. Se controla 100 % con el teclado, entra y sale como cualquier programa de terminal (`Ctrl+C`, `q`) y siempre deja la terminal limpia al cerrar.

## Características

- Ciclos: trabajo (25 min) → descanso corto (5 min) → … → descanso largo (15 min) cada 4 pomodoros.
- Duraciones configurables por archivo y por flags de CLI.
- Barra de progreso, tiempo restante, contador de pomodoros completados.
- Atajos de teclado y combinaciones (`Ctrl+…`).
- Salida segura: `Ctrl+C`, `q`, `Ctrl+D`, `SIGTERM` y panics restauran la terminal.
- Aviso al terminar una fase (campana de terminal `\x07`, notificación de escritorio opcional).

## Librerías

| Crate | Para qué |
|---|---|
| `ratatui` | Dibujar la interfaz (widgets, layout, gauge) |
| `crossterm` | Backend de ratatui: modo raw, pantalla alterna, eventos de teclado |
| `clap` (derive) | Argumentos de línea de comandos |
| `serde` + `toml` | Leer/escribir la configuración |
| `directories` | Ruta estándar del archivo de configuración por sistema operativo |
| `anyhow` | Manejo de errores simple en `main` y en infraestructura |
| `signal-hook` | Capturar `SIGTERM`/`SIGHUP` (Unix) y salir limpio |
| `notify-rust` (opcional) | Notificaciones de escritorio |

```toml
# Cargo.toml
[package]
name = "pomo"
version = "0.1.0"
edition = "2021"

[dependencies]
ratatui = "0.29"
crossterm = "0.28"
clap = { version = "4", features = ["derive"] }
serde = { version = "1", features = ["derive"] }
toml = "0.8"
directories = "5"
anyhow = "1"
signal-hook = "0.3"
notify-rust = { version = "4", optional = true }

[features]
notify = ["dep:notify-rust"]
```

> Verifica las últimas versiones con `cargo add` antes de fijarlas; ratatui y crossterm deben ser compatibles entre sí.

## Estructura del proyecto

```
pomo/
├── Cargo.toml
├── README.md
├── AGENTS.md
└── src/
    ├── main.rs          # Punto de entrada: CLI, config, terminal, bucle principal
    ├── cli.rs           # Definición de argumentos con clap
    ├── config.rs        # Carga/guardado de configuración (TOML)
    ├── domain/
    │   ├── mod.rs
    │   ├── phase.rs     # Enum Phase { Work, ShortBreak, LongBreak }
    │   └── timer.rs     # Máquina de estados del pomodoro (sin I/O)
    ├── app.rs           # Estado de la aplicación y manejo de Actions
    ├── input.rs         # Traduce KeyEvent → Action (mapa de atajos)
    ├── ui.rs            # Render con ratatui (layout, gauge, ayuda)
    ├── terminal.rs      # Init/restore de terminal + panic hook + señales
    └── notify.rs        # Campana y notificación de escritorio
```

## Cómo se construye cada parte

### 1. `domain/phase.rs` — Fases
Enum `Phase` con `Work`, `ShortBreak`, `LongBreak`. Métodos: `label()` (texto para mostrar) y `is_break()`.

### 2. `domain/timer.rs` — Lógica del pomodoro (núcleo)
Estructura pura, sin terminal ni reloj global, para poder probarla con tests.

Campos principales:
- `phase: Phase`
- `status: Running | Paused | Idle`
- `remaining: Duration`
- `completed_pomodoros: u32`
- `durations` (work, short, long) y `long_break_every` (4)
- `last_tick: Option<Instant>`

Métodos:
- `start()`, `pause()`, `toggle()`, `reset()` (reinicia la fase actual).
- `skip()` pasa a la siguiente fase.
- `tick(now: Instant) -> Option<PhaseFinished>`: resta `now - last_tick` a `remaining`. **Usar `Instant` y no contar ticks**, así el tiempo no se desfasa si el bucle se retrasa.
- `adjust(delta_minutes: i32)`: suma/resta minutos a la fase actual (mínimo 1).
- Al llegar a cero: si era `Work`, incrementa `completed_pomodoros` y decide descanso corto o largo (`completed % long_break_every == 0`); si era descanso, vuelve a `Work`. Devuelve un evento para que `app` avise.

### 3. `config.rs` — Configuración
```toml
# ~/.config/pomo/config.toml (Linux), equivalente en macOS/Windows
work_minutes = 25
short_break_minutes = 5
long_break_minutes = 15
long_break_every = 4
auto_start_next = false
bell = true
desktop_notifications = false
```
- Ruta con `directories::ProjectDirs`.
- Si no existe, usar `Default` (no fallar).
- Los flags de CLI sobrescriben el archivo.

### 4. `cli.rs` — Argumentos
```
pomo [OPTIONS]
  -w, --work <MIN>          Minutos de trabajo
  -s, --short <MIN>         Minutos de descanso corto
  -l, --long <MIN>          Minutos de descanso largo
  -c, --cycles <N>          Pomodoros antes del descanso largo
      --auto-start          Iniciar la siguiente fase sola
      --no-bell             Sin campana
  -h, --help / -V, --version
```

### 5. `terminal.rs` — Terminal segura
Es la parte clave para que se comporte como un programa de terminal normal.

- `init()`: `enable_raw_mode()` + `EnterAlternateScreen` + ocultar cursor; devuelve `Terminal<CrosstermBackend<Stdout>>`.
- `restore()`: salir de pantalla alterna, mostrar cursor, `disable_raw_mode()`. Debe ser idempotente.
- **Panic hook**: guardar el hook anterior, instalar uno nuevo que llame `restore()` y luego el anterior. Así un panic no deja la terminal rota.
- **Señales**: con `signal-hook` registrar `SIGTERM` y `SIGHUP` en un `AtomicBool`; el bucle principal lo revisa y sale.
- Importante: en modo raw, `Ctrl+C` **no** envía `SIGINT`; llega como un `KeyEvent` con `KeyCode::Char('c')` + `KeyModifiers::CONTROL`, así que hay que manejarlo en `input.rs`.
- Opcional: `Ctrl+Z` para suspender (restaurar terminal, `raise(SIGTSTP)`, reinicializar al volver con `SIGCONT`).

### 6. `input.rs` — Atajos de teclado
Función `map_key(KeyEvent) -> Option<Action>`. Filtrar `key.kind == KeyEventKind::Press` (en Windows llegan también `Release`/`Repeat`).

`Action`: `Quit`, `ToggleTimer`, `Reset`, `Skip`, `Increase`, `Decrease`, `ToggleHelp`, `SelectPhase(Phase)`, `Redraw`, `ToggleBell`.

| Atajo | Acción |
|---|---|
| `Ctrl+C` | Salir |
| `Ctrl+D` | Salir |
| `q` / `Esc` | Salir (Esc cierra primero la ayuda si está abierta) |
| `Space` / `Enter` | Iniciar / pausar |
| `r` / `Ctrl+R` | Reiniciar fase actual |
| `n` / `s` / `→` | Saltar a la siguiente fase |
| `+` / `↑` | +1 minuto a la fase actual |
| `-` / `↓` | −1 minuto a la fase actual |
| `1` / `2` / `3` | Ir a Trabajo / Descanso corto / Descanso largo |
| `b` | Activar/desactivar campana |
| `?` / `h` / `F1` | Mostrar/ocultar ayuda |
| `Ctrl+L` | Redibujar pantalla |

Ejemplo del patrón:
```rust
match (key.code, key.modifiers) {
    (KeyCode::Char('c'), m) if m.contains(KeyModifiers::CONTROL) => Some(Action::Quit),
    (KeyCode::Char('d'), m) if m.contains(KeyModifiers::CONTROL) => Some(Action::Quit),
    (KeyCode::Char(' '), _) | (KeyCode::Enter, _) => Some(Action::ToggleTimer),
    (KeyCode::Char('q'), _) => Some(Action::Quit),
    // ...
    _ => None,
}
```

### 7. `app.rs` — Estado de la aplicación
```rust
pub struct App {
    pub timer: Timer,
    pub show_help: bool,
    pub bell: bool,
    pub should_quit: bool,
    pub message: Option<(String, Instant)>, // aviso temporal en la barra inferior
}
```
- `handle(action)`: aplica la acción al `timer` o al estado de la UI.
- `on_tick(now)`: llama `timer.tick(now)`; si una fase terminó, dispara `notify` y, si `auto_start_next`, arranca la siguiente.

### 8. `ui.rs` — Interfaz
Layout vertical con `Layout::vertical`:

1. **Cabecera**: nombre de la app y fase actual (color por fase: rojo trabajo, verde descanso corto, azul descanso largo).
2. **Centro**: tiempo restante grande `MM:SS` (centrado) y un `Gauge` con el progreso de la fase.
3. **Estado**: `▶ En marcha` / `⏸ Pausado`, pomodoros completados (`🍅 × 3` o `●●●○`).
4. **Pie**: resumen de atajos (`Space pausa · r reiniciar · n saltar · ? ayuda · q salir`).
5. **Popup de ayuda** (`Clear` + `Block` centrado) con la tabla completa de atajos cuando `show_help` es `true`.

Debe adaptarse a terminales pequeñas (mínimo ~30×8); si no cabe, mostrar solo el tiempo.

### 9. `notify.rs` — Avisos
- Campana: escribir `\x07` a stdout.
- Notificación de escritorio con `notify-rust` detrás de `#[cfg(feature = "notify")]`; los errores se ignoran (no deben romper la app).

### 10. `main.rs` — Bucle principal
```rust
fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let config = Config::load()?.merge(cli);
    terminal::install_panic_hook();
    let signals = terminal::register_signals()?;
    let mut term = terminal::init()?;

    let mut app = App::new(config);
    let tick_rate = Duration::from_millis(200);

    while !app.should_quit && !signals.load(Ordering::Relaxed) {
        term.draw(|f| ui::render(f, &app))?;

        if event::poll(tick_rate)? {
            if let Event::Key(key) = event::read()? {
                if let Some(action) = input::map_key(key) {
                    app.handle(action);
                }
            }
        }
        app.on_tick(Instant::now());
    }

    terminal::restore()?;
    Ok(())
}
```
Notas:
- `event::poll(tick_rate)` evita consumir CPU y actualiza el reloj ~5 veces por segundo.
- Manejar `Event::Resize` simplemente redibujando.
- Siempre llamar `restore()` también si el bucle devuelve error (usar un guard con `Drop` o `let result = run(); restore(); result`).

## Uso

```bash
cargo run --release                 # 25/5/15 por defecto
cargo run --release -- -w 50 -s 10  # 50 min de trabajo, 10 de descanso
cargo run --release --features notify
cargo install --path .              # instalar como comando `pomo`
```

## Pruebas

- `domain/timer.rs`: tests unitarios de transiciones (trabajo→corto, 4.º pomodoro→largo, `skip`, `adjust` con mínimo 1, `reset`). Se pasa un `Instant` simulado a `tick`, sin dormir.
- `input.rs`: tests de `map_key` para cada atajo, incluido `Ctrl+C`.
- `cargo clippy -- -D warnings` y `cargo fmt --check` antes de cada commit.

## Orden de implementación sugerido

1. `cargo new pomo` y dependencias.
2. `domain/` con tests (sin UI).
3. `terminal.rs` + `main.rs` con un bucle que solo muestre texto y salga con `q`/`Ctrl+C`.
4. `input.rs` + `app.rs` (atajos conectados al timer).
5. `ui.rs` (layout, gauge, ayuda).
6. `config.rs` + `cli.rs`.
7. `notify.rs` y señales.
8. Pulido: terminales pequeñas, colores, mensajes temporales.

## Ideas futuras

- Estadísticas diarias guardadas en archivo (pomodoros por día).
- Etiquetas/tareas por pomodoro.
- Sonido con `rodio`.
- Temas de color.