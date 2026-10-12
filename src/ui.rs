use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Clear, Gauge, Paragraph, Row, Table};
use ratatui::Frame;

use crate::app::App;
use crate::domain::{Phase, Status};

// Usa los colores ANSI del tema activo en la terminal (respeta Catppuccin,
// Gruvbox, Tokyo Night, etc. configurados en Omarchy/Alacritty).
fn phase_color(phase: Phase) -> Color {
    match phase {
        Phase::Work => Color::Red,
        Phase::ShortBreak => Color::Green,
        Phase::LongBreak => Color::Blue,
    }
}

fn phase_icon(phase: Phase) -> &'static str {
    match phase {
        Phase::Work => "🍅",
        Phase::ShortBreak => "☕",
        Phase::LongBreak => "🌿",
    }
}

// Dígitos grandes 5×3 con bordes dobles
fn digit_5x3(c: char) -> [&'static str; 5] {
    match c {
        '0' => ["╔═══╗", "║   ║", "║   ║", "║   ║", "╚═══╝"],
        '1' => ["  ╗  ", "  ║  ", "  ║  ", "  ║  ", "  ╩  "],
        '2' => ["╔═══╗", "    ║", "╔═══╝", "║    ", "╚═══╝"],
        '3' => ["╔═══╗", "    ║", " ═══╣", "    ║", "╚═══╝"],
        '4' => ["║   ║", "║   ║", "╚═══╣", "    ║", "    ╩"],
        '5' => ["╔════", "║    ", "╚═══╗", "    ║", "╚═══╝"],
        '6' => ["╔════", "║    ", "╠═══╗", "║   ║", "╚═══╝"],
        '7' => ["╔═══╗", "    ║", "    ║", "    ║", "    ╩"],
        '8' => ["╔═══╗", "║   ║", "╠═══╣", "║   ║", "╚═══╝"],
        '9' => ["╔═══╗", "║   ║", "╚═══╣", "    ║", "╚═══╝"],
        ':' => ["     ", "  ●  ", "     ", "  ●  ", "     "],
        _ => ["     ", "     ", "     ", "     ", "     "],
    }
}

pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();

    if area.width < 32 || area.height < 8 {
        render_minimal(frame, app, area);
    } else {
        render_full(frame, app, area);
    }

    if app.show_help {
        render_help_popup(frame, area);
    }
}

fn render_minimal(frame: &mut Frame, app: &App, area: Rect) {
    let mins = app.timer.remaining().as_secs() / 60;
    let secs = app.timer.remaining().as_secs() % 60;
    let color = phase_color(app.timer.phase());

    let status_sym = match app.timer.status() {
        Status::Running => "▶",
        Status::Paused => "⏸",
        Status::Idle => "·",
    };

    let lines = vec![
        Line::from(Span::styled(
            format!("{:02}:{:02}", mins, secs),
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            format!("{} {}", status_sym, app.timer.phase().label()),
            Style::default().fg(color),
        )),
    ];

    frame.render_widget(Paragraph::new(lines).alignment(Alignment::Center), area);
}

fn render_full(frame: &mut Frame, app: &App, area: Rect) {
    let color = phase_color(app.timer.phase());

    let main_chunks = Layout::vertical([
        Constraint::Length(3), // Header
        Constraint::Min(7),    // Centro (tiempo + barra)
        Constraint::Length(3), // Estado + pomodoros
        Constraint::Length(1), // Footer
    ])
    .split(area);

    render_header(frame, app, main_chunks[0], color);
    render_center(frame, app, main_chunks[1], color);
    render_status(frame, app, main_chunks[2], color);
    render_footer(frame, app, main_chunks[3]);
}

fn render_header(frame: &mut Frame, app: &App, area: Rect, color: Color) {
    let block = Block::default()
        .borders(Borders::BOTTOM)
        .border_type(BorderType::Plain)
        // Color::DarkGray es el color #8 del tema: se adapta solo
        .border_style(Style::default().fg(Color::DarkGray));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let completed = app.timer.completed_pomodoros();
    let line = Line::from(vec![
        Span::styled(" pomo ", Style::default().add_modifier(Modifier::BOLD)),
        Span::styled("·", Style::default().fg(Color::DarkGray)),
        Span::raw(" "),
        Span::raw(phase_icon(app.timer.phase())),
        Span::raw(" "),
        Span::styled(
            app.timer.phase().label(),
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled(
            format!("({})", completed),
            Style::default().fg(Color::DarkGray),
        ),
    ]);

    frame.render_widget(Paragraph::new(line).alignment(Alignment::Center), inner);
}

fn render_center(frame: &mut Frame, app: &App, area: Rect, color: Color) {
    let mins = app.timer.remaining().as_secs() / 60;
    let secs = app.timer.remaining().as_secs() % 60;
    let use_big_digits = area.height >= 9 && area.width >= 52 && mins <= 99;

    let total_secs = app.timer.total_duration().as_secs_f64();
    let remaining_secs = app.timer.remaining().as_secs_f64();
    let elapsed = (total_secs - remaining_secs).max(0.0);
    let ratio = if total_secs > 0.0 {
        (elapsed / total_secs).clamp(0.0, 1.0)
    } else {
        0.0
    };

    let time_height = if use_big_digits { 5 } else { 1 };
    let gauge_height = if area.height >= 10 { 3 } else { 1 };
    let spacer = if area.height >= 12 { 1 } else { 0 };

    let center_chunks = Layout::vertical([
        Constraint::Min(time_height),
        Constraint::Length(spacer),
        Constraint::Length(gauge_height),
    ])
    .split(area);

    // Tiempo grande
    if use_big_digits {
        let time_str = format!("{:02}:{:02}", mins, secs);
        let chars: Vec<char> = time_str.chars().collect();
        let g: Vec<[&'static str; 5]> = chars.iter().map(|&c| digit_5x3(c)).collect();
        let lines: Vec<Line> = (0..5)
            .map(|row| {
                let row_str = g
                    .iter()
                    .map(|glyph| glyph[row])
                    .collect::<Vec<_>>()
                    .join(" ");
                Line::from(Span::styled(
                    row_str,
                    Style::default().fg(color).add_modifier(Modifier::BOLD),
                ))
            })
            .collect();
        frame.render_widget(
            Paragraph::new(lines).alignment(Alignment::Center),
            center_chunks[0],
        );
    } else {
        let time_text = format!("{:02}:{:02}", mins, secs);
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                time_text,
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            )))
            .alignment(Alignment::Center),
            center_chunks[0],
        );
    }

    // Gauge
    let percent = (ratio * 100.0) as u16;
    let gauge = if gauge_height >= 3 {
        Gauge::default()
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(Color::DarkGray)),
            )
            .gauge_style(Style::default().fg(color).add_modifier(Modifier::BOLD))
            .ratio(ratio)
            .label(format!(" {}% ", percent))
    } else {
        Gauge::default()
            .gauge_style(Style::default().fg(color))
            .ratio(ratio)
            .label("")
    };
    frame.render_widget(gauge, center_chunks[2]);
}

fn render_status(frame: &mut Frame, app: &App, area: Rect, color: Color) {
    let block = Block::default()
        .borders(Borders::TOP)
        .border_type(BorderType::Plain)
        .border_style(Style::default().fg(Color::DarkGray));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let (sym, label, status_color) = match app.timer.status() {
        Status::Running => ("▶", " Corriendo", Color::Green),
        Status::Paused => ("⏸", " Pausado  ", Color::Yellow),
        Status::Idle => ("·", " Inactivo ", Color::DarkGray),
    };

    let every = app.timer.long_break_every().max(1);
    let completed = app.timer.completed_pomodoros();
    let in_cycle = completed % every;
    let dots: String = (0..every)
        .map(|i| if i < in_cycle { '█' } else { '░' })
        .collect();

    let halves =
        Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]).split(inner);

    let mut status_spans = vec![
        Span::styled(
            format!(" {} ", sym),
            Style::default()
                .fg(status_color)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(label, Style::default().fg(status_color)),
    ];
    if !app.sound {
        status_spans.push(Span::styled(" · 🔇", Style::default().fg(Color::DarkGray)));
    }

    frame.render_widget(
        Paragraph::new(Line::from(status_spans)).alignment(Alignment::Center),
        halves[0],
    );

    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(
                format!("🍅 ×{}  ", completed),
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            ),
            Span::styled(format!("[{}]", dots), Style::default().fg(Color::DarkGray)),
        ]))
        .alignment(Alignment::Center),
        halves[1],
    );
}

fn render_footer(frame: &mut Frame, app: &App, area: Rect) {
    let line = if let Some((msg, _)) = &app.message {
        Line::from(vec![
            Span::styled(
                "🔔 ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(msg.as_str(), Style::default().fg(Color::Yellow)),
        ])
    } else {
        Line::from(vec![
            key_hint("Space"),
            Span::styled(" pausa  ", Style::default().fg(Color::DarkGray)),
            key_hint("r"),
            Span::styled(" reiniciar  ", Style::default().fg(Color::DarkGray)),
            key_hint("n"),
            Span::styled(" saltar  ", Style::default().fg(Color::DarkGray)),
            key_hint("+/-"),
            Span::styled(" ajustar  ", Style::default().fg(Color::DarkGray)),
            key_hint("?"),
            Span::styled(" ayuda  ", Style::default().fg(Color::DarkGray)),
            key_hint("q"),
            Span::styled(" salir", Style::default().fg(Color::DarkGray)),
        ])
    };

    frame.render_widget(Paragraph::new(line).alignment(Alignment::Center), area);
}

fn key_hint(key: &str) -> Span<'_> {
    Span::styled(
        format!("[{}]", key),
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    )
}

fn centered_rect(width: u16, height: u16, r: Rect) -> Rect {
    let x = r.x.saturating_add(r.width.saturating_sub(width) / 2);
    let y = r.y.saturating_add(r.height.saturating_sub(height) / 2);
    Rect {
        x: x.min(r.right().saturating_sub(1)),
        y: y.min(r.bottom().saturating_sub(1)),
        width: width.min(r.width),
        height: height.min(r.height),
    }
}

fn render_help_popup(frame: &mut Frame, area: Rect) {
    let popup_width = 62u16.min(area.width);
    let popup_height = 17u16.min(area.height);
    let popup_area = centered_rect(popup_width, popup_height, area);

    let rows = vec![
        Row::new(vec!["Ctrl+C / Ctrl+D / q", "Salir"]),
        Row::new(vec!["Esc", "Cerrar ayuda / Salir"]),
        Row::new(vec!["Space / Enter", "Iniciar / Pausar"]),
        Row::new(vec!["r / Ctrl+R", "Reiniciar fase actual"]),
        Row::new(vec!["n / s / →", "Saltar a la siguiente fase"]),
        Row::new(vec!["+  /  ↑", "+1 minuto"]),
        Row::new(vec!["-  /  ↓", "-1 minuto"]),
        Row::new(vec!["1 / 2 / 3", "Trabajo / Descanso corto / Largo"]),
        Row::new(vec!["b", "Activar/desactivar campana"]),
        Row::new(vec!["m", "Activar/desactivar sonido ambiental"]),
        Row::new(vec!["? / h / F1", "Mostrar/ocultar ayuda"]),
        Row::new(vec!["Ctrl+L", "Redibujar pantalla"]),
    ];

    let table = Table::new(rows, [Constraint::Length(22), Constraint::Min(28)])
        .header(
            Row::new(vec!["  Atajo", "Acción"]).style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::DarkGray))
                .title(Span::styled(
                    " ⌨  Atajos · Esc para cerrar ",
                    Style::default().add_modifier(Modifier::BOLD),
                )),
        )
        .column_spacing(2);

    frame.render_widget(Clear, popup_area);
    frame.render_widget(table, popup_area);
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    #[test]
    fn test_render_small_size_20x5() {
        let backend = TestBackend::new(20, 5);
        let mut terminal = Terminal::new(backend).expect("crear terminal de prueba");
        let app = App::default();
        terminal
            .draw(|f| render(f, &app))
            .expect("renderizar en 20x5 sin panic");
    }

    #[test]
    fn test_render_medium_size() {
        let backend = TestBackend::new(40, 12);
        let mut terminal = Terminal::new(backend).expect("crear terminal de prueba");
        let app = App::default();
        terminal
            .draw(|f| render(f, &app))
            .expect("renderizar en tamaño medio");
    }

    #[test]
    fn test_render_standard_size() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).expect("crear terminal de prueba");
        let mut app = App::default();
        terminal
            .draw(|f| render(f, &app))
            .expect("renderizar en tamaño estándar");

        app.show_help = true;
        terminal
            .draw(|f| render(f, &app))
            .expect("renderizar ayuda");

        app.set_message("Test alerta");
        terminal
            .draw(|f| render(f, &app))
            .expect("renderizar aviso");
    }

    #[test]
    fn test_render_all_phases() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).expect("crear terminal de prueba");
        let mut app = App::default();
        for phase in [Phase::Work, Phase::ShortBreak, Phase::LongBreak] {
            app.timer.select_phase(phase);
            terminal
                .draw(|f| render(f, &app))
                .expect("renderizar fases");
        }
    }

    #[test]
    fn test_render_large_size() {
        let backend = TestBackend::new(120, 36);
        let mut terminal = Terminal::new(backend).expect("crear terminal de prueba");
        let app = App::default();
        terminal
            .draw(|f| render(f, &app))
            .expect("renderizar en pantalla grande");
    }
}
