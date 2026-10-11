use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Gauge, Paragraph, Row, Table};
use ratatui::Frame;

use crate::app::App;
use crate::domain::{Phase, Status};

pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();

    // En pantallas muy pequeñas, mostrar vista mínima para evitar rotura o panic
    if area.width < 32 || area.height < 8 {
        render_minimal(frame, app, area);
        return;
    }

    render_full(frame, app, area);

    if app.show_help {
        render_help_popup(frame, area);
    }
}

fn phase_color(phase: Phase) -> Color {
    match phase {
        Phase::Work => Color::Red,
        Phase::ShortBreak => Color::Green,
        Phase::LongBreak => Color::Cyan,
    }
}

fn digit_3x3(c: char) -> [&'static str; 3] {
    match c {
        '0' => ["┌─┐", "│ │", "└─┘"],
        '1' => [" ┐ ", " │ ", " ┴ "],
        '2' => ["┌─┐", "┌─┘", "└──"],
        '3' => ["┌─┐", " ─┤", "──┘"],
        '4' => ["│ │", "└─┤", "  │"],
        '5' => ["┌──", "└─┐", "──┘"],
        '6' => ["┌──", "├─┐", "└─┘"],
        '7' => ["──┐", "  │", "  │"],
        '8' => ["┌─┐", "├─┤", "└─┘"],
        '9' => ["┌─┐", "└─┤", "──┘"],
        ':' => ["   ", " · ", "   "],
        _ => ["   ", "   ", "   "],
    }
}

fn render_minimal(frame: &mut Frame, app: &App, area: Rect) {
    let mins = app.timer.remaining().as_secs() / 60;
    let secs = app.timer.remaining().as_secs() % 60;
    let color = phase_color(app.timer.phase());

    let status_sym = match app.timer.status() {
        Status::Running => "▶",
        Status::Paused => "⏸",
        Status::Idle => "⏹",
    };

    let lines = vec![
        Line::from(Span::styled(
            format!("{:02}:{:02}", mins, secs),
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::raw(format!(
            "{} {}",
            status_sym,
            app.timer.phase().label()
        ))),
    ];

    let paragraph = Paragraph::new(lines).alignment(Alignment::Center);
    frame.render_widget(paragraph, area);
}

fn render_full(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::vertical([
        Constraint::Length(3), // 1. Cabecera
        Constraint::Min(4),    // 2. Centro (tiempo grande + gauge)
        Constraint::Length(2), // 3. Estado y pomodoros
        Constraint::Length(1), // 4. Pie
    ])
    .split(area);

    let color = phase_color(app.timer.phase());

    // 1. Cabecera
    let header_text = Line::from(vec![
        Span::styled(" pomo ", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw("— "),
        Span::styled(
            format!(" {} ", app.timer.phase().label()),
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        ),
    ]);
    let header = Paragraph::new(header_text)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::BOTTOM));
    frame.render_widget(header, chunks[0]);

    // 2. Centro: Tiempo + Gauge
    let center_area = chunks[1];
    let mins = app.timer.remaining().as_secs() / 60;
    let secs = app.timer.remaining().as_secs() % 60;

    let use_big_digits = center_area.height >= 6 && center_area.width >= 35 && mins <= 99;

    let center_chunks = Layout::vertical([
        Constraint::Min(3),                                              // Tiempo
        Constraint::Length(if center_area.height >= 8 { 3 } else { 1 }), // Gauge
    ])
    .split(center_area);

    if use_big_digits {
        let time_str = format!("{:02}:{:02}", mins, secs);
        let chars: Vec<char> = time_str.chars().collect();
        let g: Vec<[&'static str; 3]> = chars.iter().map(|&c| digit_3x3(c)).collect();

        let l0 = format!(
            "{} {} {} {} {}",
            g[0][0], g[1][0], g[2][0], g[3][0], g[4][0]
        );
        let l1 = format!(
            "{} {} {} {} {}",
            g[0][1], g[1][1], g[2][1], g[3][1], g[4][1]
        );
        let l2 = format!(
            "{} {} {} {} {}",
            g[0][2], g[1][2], g[2][2], g[3][2], g[4][2]
        );

        let time_lines = vec![
            Line::from(Span::styled(
                l0,
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::styled(
                l1,
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::styled(
                l2,
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            )),
        ];
        let time_widget = Paragraph::new(time_lines).alignment(Alignment::Center);
        frame.render_widget(time_widget, center_chunks[0]);
    } else {
        let time_text = format!("{:02}:{:02}", mins, secs);
        let time_widget = Paragraph::new(Line::from(Span::styled(
            time_text,
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        )))
        .alignment(Alignment::Center);
        frame.render_widget(time_widget, center_chunks[0]);
    }

    // Gauge de progreso
    let total_secs = app.timer.total_duration().as_secs_f64();
    let remaining_secs = app.timer.remaining().as_secs_f64();
    let elapsed_secs = (total_secs - remaining_secs).max(0.0);
    let ratio = if total_secs > 0.0 {
        (elapsed_secs / total_secs).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let percent = (ratio * 100.0) as u16;

    let mut gauge = Gauge::default()
        .gauge_style(Style::default().fg(color))
        .ratio(ratio)
        .label(format!("{}%", percent));

    if center_area.height >= 8 {
        gauge = gauge.block(Block::default().borders(Borders::ALL).title(" Progreso "));
    }
    frame.render_widget(gauge, center_chunks[1]);

    // 3. Estado y Pomodoros
    let status_span = match app.timer.status() {
        Status::Running => Span::styled(
            "▶ En marcha",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ),
        Status::Paused => Span::styled(
            "⏸ Pausado",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Status::Idle => Span::styled("⏹ Inactivo", Style::default().fg(Color::DarkGray)),
    };

    let completed = app.timer.completed_pomodoros();
    let every = app.timer.long_break_every().max(1);
    let in_cycle = completed % every;
    let dots: String = (0..every)
        .map(|i| if i < in_cycle { '●' } else { '○' })
        .collect();

    let status_line = Line::from(vec![
        status_span,
        Span::raw("   ·   "),
        Span::raw(format!("🍅 × {}  [{}]", completed, dots)),
    ]);
    let status_widget = Paragraph::new(status_line).alignment(Alignment::Center);
    frame.render_widget(status_widget, chunks[2]);

    // 4. Pie
    let footer_line = if let Some((msg, _)) = &app.message {
        Line::from(vec![
            Span::styled(
                "🔔 ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(msg, Style::default().fg(Color::Yellow)),
        ])
    } else {
        Line::from(vec![
            Span::styled("Space", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" pausa · "),
            Span::styled("r", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" reiniciar · "),
            Span::styled("n", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" saltar · "),
            Span::styled("?", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" ayuda · "),
            Span::styled("q", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" salir"),
        ])
    };
    let footer_widget = Paragraph::new(footer_line).alignment(Alignment::Center);
    frame.render_widget(footer_widget, chunks[3]);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::vertical([
        Constraint::Percentage((100 - percent_y) / 2),
        Constraint::Percentage(percent_y),
        Constraint::Percentage((100 - percent_y) / 2),
    ])
    .split(r);

    Layout::horizontal([
        Constraint::Percentage((100 - percent_x) / 2),
        Constraint::Percentage(percent_x),
        Constraint::Percentage((100 - percent_x) / 2),
    ])
    .split(popup_layout[1])[1]
}

fn render_help_popup(frame: &mut Frame, area: Rect) {
    let popup_area = centered_rect(75, 75, area);

    let rows = vec![
        Row::new(vec!["Ctrl+C, Ctrl+D, q", "Salir"]),
        Row::new(vec!["Esc", "Cerrar ayuda / Salir"]),
        Row::new(vec!["Space, Enter", "Iniciar / pausar temporizador"]),
        Row::new(vec!["r, Ctrl+R", "Reiniciar fase actual"]),
        Row::new(vec!["n, s, →", "Saltar a la siguiente fase"]),
        Row::new(vec!["+, ↑", "+1 minuto a la fase actual"]),
        Row::new(vec!["-, ↓", "-1 minuto a la fase actual"]),
        Row::new(vec![
            "1, 2, 3",
            "Ir a Trabajo / Descanso corto / Descanso largo",
        ]),
        Row::new(vec!["b", "Activar / desactivar campana"]),
        Row::new(vec!["?, h, F1", "Mostrar / ocultar ayuda"]),
        Row::new(vec!["Ctrl+L", "Redibujar pantalla"]),
    ];

    let table = Table::new(rows, [Constraint::Length(22), Constraint::Min(24)])
        .header(
            Row::new(vec!["Atajo", "Acción"]).style(
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(Color::Yellow),
            ),
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Ayuda — Atajos de teclado (Esc o ? para cerrar) "),
        );

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

        // Con ayuda abierta
        app.show_help = true;
        terminal
            .draw(|f| render(f, &app))
            .expect("renderizar ayuda");

        // Con mensaje activo
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
}
