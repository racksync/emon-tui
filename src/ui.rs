use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Axis, Block, BorderType, Borders, Chart, Clear, Dataset, Gauge, Padding, Paragraph, Row,
        Table,
    },
    Frame,
};

use crate::state::AppState;

pub fn render(f: &mut Frame, app: &AppState) {
    let size = f.area();

    // Minimum terminal size check
    const MIN_WIDTH: u16 = 60;
    const MIN_HEIGHT: u16 = 25;

    if size.width < MIN_WIDTH || size.height < MIN_HEIGHT {
        render_terminal_too_small(f, size, MIN_WIDTH, MIN_HEIGHT);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),      // Header
            Constraint::Percentage(9),  // Gauges
            Constraint::Percentage(44), // Data tables + Daily energy (combined)
            Constraint::Percentage(43), // Charts
            Constraint::Length(1),      // Footer
        ])
        .split(size);

    render_header(f, app, chunks[0]);
    render_main_gauges(f, app, chunks[1]);
    render_combined_tables(f, app, chunks[2]); // New combined layout
    render_charts(f, app, chunks[3]);
    render_footer(f, chunks[4]);

    // Render error popup if there's an error
    if app.error.is_some() {
        render_error_popup(f, app, size);
    }
}

fn render_header(f: &mut Frame, app: &AppState, area: Rect) {
    use chrono::Utc;
    use chrono_tz::Tz;

    // Calculate elapsed time with millisecond precision
    let elapsed_ms = app
        .last_fetch
        .map(|t| t.elapsed().as_millis())
        .unwrap_or(999000);
    let elapsed_secs = elapsed_ms as f64 / 1000.0;

    let (status_text, status_color) = if app.error.is_some() {
        ("RECONNECTING", Color::LightYellow)
    } else if elapsed_ms < 5000 {
        ("LIVE", Color::LightGreen)
    } else if elapsed_ms < 10000 {
        ("SYNC", Color::LightYellow)
    } else {
        ("STALE", Color::Gray)
    };

    let spinner = match (std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis()
        / 250)
        % 4
    {
        0 => "◴",
        1 => "◷",
        2 => "◶",
        _ => "◵",
    };

    // Get current time in configured timezone with UTC offset format
    let tz_str = app
        .config
        .home_assistant
        .timezone
        .as_deref()
        .unwrap_or("Asia/Bangkok");
    let tz: Tz = tz_str.parse().unwrap_or(chrono_tz::Asia::Bangkok);
    let now_utc = Utc::now();
    let now_local = now_utc.with_timezone(&tz);

    // Format: 2026-01-22 17:30:45 UTC+07
    let time_str = now_local.format("%Y-%m-%d %H:%M:%S UTC%:z").to_string();

    // Get refresh rate from config
    let refresh_rate = app
        .config
        .home_assistant
        .fetch_interval_seconds
        .unwrap_or(5);
    let refresh_str = if refresh_rate == 0 {
        "Refresh: realtime".to_string()
    } else {
        format!("Refresh: {}s", refresh_rate)
    };

    // Build the second line conditionally based on refresh rate
    let mut second_line_spans = vec![
        Span::raw("  "), // 2 spaces to align "Connected to:" with "Real-time Energy Monitor" (after emoji removal)
    ];

    // Add config path if custom config is used
    if let Some(ref config_path) = app.config_path {
        second_line_spans.push(Span::raw("  ")); // 2 spaces to align with "Connected to:"
        second_line_spans.push(Span::styled(
            format!("Config: {}", config_path.display()),
            Style::default()
                .fg(Color::Rgb(255, 200, 100))
                .add_modifier(Modifier::BOLD),
        ));
    }

    second_line_spans.push(Span::raw("    "));
    second_line_spans.push(Span::styled(
        "Connected to: ",
        Style::default().fg(Color::Rgb(150, 150, 150)),
    ));
    second_line_spans.push(Span::styled(
        app.config.home_assistant.url.as_str(),
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    ));
    second_line_spans.push(Span::raw("  |  "));
    second_line_spans.push(Span::styled(
        &refresh_str,
        Style::default().fg(Color::Rgb(100, 200, 255)),
    ));

    // Only show countdown timer if NOT in realtime mode (refresh_rate != 0)
    if refresh_rate != 0 {
        second_line_spans.push(Span::raw("  |  "));
        second_line_spans.push(Span::styled(
            if elapsed_secs < 1.0 {
                format!("Update: {:.1}s ago", elapsed_secs)
            } else {
                format!("Update: {:.0}s ago", elapsed_secs)
            },
            Style::default().fg(Color::Rgb(120, 120, 120)),
        ));
    }

    let header = Paragraph::new(vec![
        Line::from(vec![
            Span::styled(
                "Real-time Energy Monitor",
                Style::default()
                    .fg(Color::LightCyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
            Span::styled(spinner, Style::default().fg(status_color)),
            Span::raw(" "),
            Span::styled(
                status_text,
                Style::default()
                    .fg(status_color)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("  |  "),
            Span::styled(&time_str, Style::default().fg(Color::Rgb(200, 200, 100))),
        ]),
        Line::from(second_line_spans),
    ])
    .alignment(Alignment::Center);

    f.render_widget(header, area);
}

fn render_main_gauges(f: &mut Frame, app: &AppState, area: Rect) {
    let latest = app.history.last();

    let gauge_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(33),
            Constraint::Percentage(33),
            Constraint::Percentage(34),
        ])
        .margin(0)
        .split(area);

    let solar_val = latest.map(|d| d.solar).unwrap_or(0.0);
    let load_val = latest.map(|d| d.load).unwrap_or(0.0);
    let battery_soc = latest.map(|d| d.battery_soc).unwrap_or(0.0);
    let load_ratio_pct = latest.map(|d| d.load_ratio).unwrap_or(0.0);

    let max_solar = 18000.0; // Fixed 18kW scale
    let max_load = 18000.0; // Fixed 18kW scale

    let solar_ratio = (solar_val / max_solar).min(1.0);
    let load_ratio = (load_val / max_load).min(1.0);
    let soc_ratio = (battery_soc / 100.0).min(1.0);

    let solar_label = format!("☀️ {:.1} kW", solar_val / 1000.0);
    let load_label = format!("🏠 {:.1} kW ({:.1}%)", load_val / 1000.0, load_ratio_pct);

    // SOC label with optional remaining capacity
    let soc_label = if let Some(capacity_kwh) = app.config.home_assistant.battery_capacity_kwh {
        let remaining_kwh = (battery_soc / 100.0) * capacity_kwh;
        format!("🔋 {:.0}% ({:.2} kWh)", battery_soc, remaining_kwh)
    } else {
        format!("🔋 {:.0}%", battery_soc)
    };

    let solar_text_color = get_solar_text_color(solar_ratio); // Dynamic text color

    let solar_gauge = Gauge::default()
        .block(
            Block::default()
                .title(" SOLAR ")
                .title_style(
                    Style::default()
                        .fg(Color::Rgb(255, 215, 0)) // Gold
                        .add_modifier(Modifier::BOLD),
                )
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Rgb(255, 200, 0))),
        )
        .gauge_style(
            Style::default()
                .fg(get_gradient_color(solar_ratio))
                .bg(Color::Rgb(20, 20, 20)), // Darker background
        )
        .percent((solar_ratio * 100.0) as u16)
        .label(Span::styled(
            solar_label,
            Style::default()
                .fg(solar_text_color) // Dynamic text color based on background
                .add_modifier(Modifier::BOLD),
        ))
        .ratio(solar_ratio);

    let load_text_color = get_load_text_color(load_ratio); // Dynamic text color

    let load_gauge = Gauge::default()
        .block(
            Block::default()
                .title(" LOAD ")
                .title_style(
                    Style::default()
                        .fg(Color::Rgb(138, 161, 255)) // Light Blue
                        .add_modifier(Modifier::BOLD),
                )
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Rgb(100, 140, 255))),
        )
        .gauge_style(
            Style::default()
                .fg(get_blue_gradient_color(load_ratio))
                .bg(Color::Rgb(20, 20, 20)), // Darker background
        )
        .percent((load_ratio * 100.0) as u16)
        .label(Span::styled(
            load_label,
            Style::default()
                .fg(load_text_color) // Dynamic text color based on background
                .add_modifier(Modifier::BOLD),
        ))
        .ratio(load_ratio);

    let soc_color = get_soc_gradient_color(soc_ratio);
    let soc_text_color = get_soc_text_color(soc_ratio); // Dynamic text color

    let soc_gauge = Gauge::default()
        .block(
            Block::default()
                .title(" ESS SOC ")
                .title_style(Style::default().fg(soc_color).add_modifier(Modifier::BOLD))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(soc_color)),
        )
        .gauge_style(
            Style::default().fg(soc_color).bg(Color::Rgb(20, 20, 20)), // Darker background
        )
        .percent((soc_ratio * 100.0) as u16)
        .label(Span::styled(
            soc_label,
            Style::default()
                .fg(soc_text_color) // Dynamic text color based on background
                .add_modifier(Modifier::BOLD),
        ))
        .ratio(soc_ratio);

    f.render_widget(solar_gauge, gauge_chunks[0]);
    f.render_widget(load_gauge, gauge_chunks[1]);
    f.render_widget(soc_gauge, gauge_chunks[2]);
}

fn render_charts(f: &mut Frame, app: &AppState, area: Rect) {
    let solar_data: Vec<(f64, f64)> = app
        .get_solar_history()
        .iter()
        .enumerate()
        .map(|(i, &v)| (i as f64, v / 1000.0)) // Convert to kW
        .collect();

    let load_data: Vec<(f64, f64)> = app
        .get_load_history()
        .iter()
        .enumerate()
        .map(|(i, &v)| (i as f64, v / 1000.0)) // Convert to kW
        .collect();

    let battery_data: Vec<(f64, f64)> = app
        .history
        .iter()
        .enumerate()
        .map(|(i, d)| (i as f64, d.battery_power / 1000.0)) // Convert to kW
        .collect();

    // Calculate dynamic Y-axis bounds with 2kW steps
    // Find min value from battery data (for discharge/negative values)
    let mut min_value = 0.0_f64;

    // Check battery data for negative values (discharge)
    for (_, y) in &battery_data {
        if *y < min_value {
            min_value = *y;
        }
    }

    // Round min_value down to nearest 2kW step (for negative values)
    let min_y = if min_value < 0.0 {
        (min_value / 2.0).floor() * 2.0
    } else {
        0.0
    };

    // Use max_solar_power from config (in Watts), convert to kW, or default to 20.0 kW
    let max_power_kw = app
        .config
        .home_assistant
        .max_solar_power
        .map(|w| w / 1000.0)
        .unwrap_or(20.0);

    // Round max_power up to nearest 2kW step
    let max_y = ((max_power_kw / 2.0).ceil() * 2.0).max(2.0);

    // Generate Y-axis labels with 2kW steps
    let mut y_labels: Vec<Span> = Vec::new();
    let mut current = min_y;
    while current <= max_y {
        y_labels.push(Span::raw(format!("{:.0}", current)));
        current += 2.0;
    }

    let history_seconds = app.get_history_seconds();
    let history_display = app.get_history_duration_display();
    let (time_value, _time_unit, time_label) = app.get_history_time_unit();
    let title = format!(" POWER HISTORY ({}) ", history_display);

    let chart = Chart::new(vec![
        Dataset::default()
            .marker(ratatui::symbols::Marker::Braille) // Use Braille for better line rendering
            .graph_type(ratatui::widgets::GraphType::Line)
            .style(Style::default().fg(Color::Rgb(255, 215, 0))) // Gold
            .data(&solar_data),
        Dataset::default()
            .marker(ratatui::symbols::Marker::Braille) // Use Braille for better line rendering
            .graph_type(ratatui::widgets::GraphType::Line)
            .style(Style::default().fg(Color::Rgb(138, 161, 255))) // Light blue
            .data(&load_data),
        Dataset::default()
            .marker(ratatui::symbols::Marker::Braille) // Use Braille for better line rendering
            .graph_type(ratatui::widgets::GraphType::Line)
            .style(Style::default().fg(Color::Rgb(100, 255, 100))) // Light green
            .data(&battery_data),
    ])
    .block(
        Block::default()
            .title(title)
            .title_style(
                Style::default()
                    .fg(Color::Rgb(255, 180, 100)) // Light orange
                    .add_modifier(Modifier::BOLD),
            )
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Rgb(200, 120, 50))),
    )
    .x_axis(
        Axis::default()
            .title(time_label)
            .style(Style::default().fg(Color::Rgb(150, 150, 150)))
            .bounds([0.0, history_seconds as f64])
            .labels(vec![
                Span::raw("0"),
                Span::raw(format!("{:.1}", time_value * 0.25)),
                Span::raw(format!("{:.1}", time_value * 0.5)),
                Span::raw(format!("{:.1}", time_value * 0.75)),
                Span::raw(format!("{:.1}", time_value)),
            ]),
    )
    .y_axis(
        Axis::default()
            .title("Power (kW)")
            .style(Style::default().fg(Color::Rgb(150, 150, 150)))
            .bounds([min_y, max_y])
            .labels(y_labels),
    );

    // Split area for chart and legend
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(10),   // Chart area
            Constraint::Length(1), // Legend area
        ])
        .split(area);

    f.render_widget(chart, chunks[0]);

    // Render legend at bottom
    let legend = Paragraph::new(Line::from(vec![
        Span::raw("  "),
        Span::styled("■", Style::default().fg(Color::Rgb(255, 215, 0))), // Gold square for Solar
        Span::styled(" Solar", Style::default().fg(Color::White)),
        Span::raw("    "),
        Span::styled("■", Style::default().fg(Color::Rgb(138, 161, 255))), // Light blue square for Load
        Span::styled(" Load", Style::default().fg(Color::White)),
        Span::raw("    "),
        Span::styled("■", Style::default().fg(Color::Rgb(100, 255, 100))), // Light green square for Battery
        Span::styled(" Battery", Style::default().fg(Color::White)),
    ]))
    .alignment(Alignment::Center);

    f.render_widget(legend, chunks[1]);
}

fn render_footer(f: &mut Frame, area: Rect) {
    let footer = Paragraph::new(Line::from(vec![
        Span::styled(" Press ", Style::default().fg(Color::Rgb(150, 150, 150))),
        Span::styled(
            "q",
            Style::default()
                .fg(Color::Rgb(255, 100, 100)) // Light red
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            " to quit  |  ",
            Style::default().fg(Color::Rgb(150, 150, 150)),
        ),
        Span::styled(
            "⚡ Real-time Energy Monitor",
            Style::default()
                .fg(Color::Rgb(255, 215, 0))
                .add_modifier(Modifier::BOLD), // Gold
        ),
        Span::styled(
            "  |  Made with ",
            Style::default().fg(Color::Rgb(150, 150, 150)),
        ),
        Span::styled(
            "❤️",
            Style::default().fg(Color::Rgb(255, 100, 100)), // Red heart
        ),
        Span::styled(
            " from Thailand by ",
            Style::default().fg(Color::Rgb(150, 150, 150)),
        ),
        Span::styled(
            "racksync",
            Style::default()
                .fg(Color::Rgb(100, 200, 255)) // Light blue
                .add_modifier(Modifier::BOLD),
        ),
    ]))
    .alignment(Alignment::Center);

    f.render_widget(footer, area);
}

fn render_combined_tables(f: &mut Frame, app: &AppState, area: Rect) {
    // Split into two columns
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .margin(0)
        .split(area);

    // Card heights (without column headers, just card title + borders):
    // Top row: max(DAILY ENERGY 6 rows, REALTIME 5 rows) + borders(2) = 8 lines
    // Bottom row: max(LIFETIME 1 row, SYSTEM STATUS 11 rows) + borders(2) = 13 lines
    // Both cards in the same horizontal row get equal height

    // Left column: DAILY ENERGY TOTALS (top) + LIFETIME & ADDITIONAL (bottom)
    let left_sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(8), Constraint::Min(13)])
        .margin(0)
        .split(columns[0]);

    // Right column: REALTIME POWER (top) + SYSTEM STATUS (bottom)
    let right_sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(8), Constraint::Min(13)])
        .margin(0)
        .split(columns[1]);

    // Render all four sections
    render_daily_energy_compact(f, app, left_sections[0]);
    render_reserved_space(f, app, left_sections[1]);
    render_realtime_power(f, app, right_sections[0]);
    render_system_status(f, app, right_sections[1]);
}

fn render_realtime_power(f: &mut Frame, app: &AppState, area: Rect) {
    let latest = app.history.last();

    let grid_import_val = latest.map(|d| d.grid_import).unwrap_or(0.0);
    let grid_export_val = latest.map(|d| d.grid_export).unwrap_or(0.0);
    let battery_power = latest.map(|d| d.battery_power).unwrap_or(0.0);
    let battery_voltage = latest.map(|d| d.battery_voltage).unwrap_or(0.0);

    let solar_trend = get_power_trend(&app.history, |d| d.solar);
    let load_trend = get_power_trend(&app.history, |d| d.load);
    let grid_import_trend = get_power_trend(&app.history, |d| d.grid_import);
    let grid_export_trend = get_power_trend(&app.history, |d| d.grid_export);

    // Check if battery is floating
    let is_floating = is_battery_floating(app, battery_voltage, battery_power);

    let solar_val_str = format!("{:.2} kW", solar_val(latest) / 1000.0);
    let load_val_str = format!("{:.2} kW", load_val(latest) / 1000.0);
    let grid_import_str = format!("{:.2} kW", grid_import_val / 1000.0);
    let grid_export_str = format!("{:.2} kW", grid_export_val / 1000.0);
    let battery_power_str = format_battery_power(battery_power, is_floating);

    let main_table = Table::new(
        vec![
            Row::new(vec![
                "  Solar Production",
                solar_val_str.as_str(),
                solar_trend,
            ])
            .style(Style::default().fg(Color::White)),
            Row::new(vec![
                "  Load Consumption",
                load_val_str.as_str(),
                load_trend,
            ])
            .style(Style::default().fg(Color::White)),
            Row::new(vec![
                "  Grid Import",
                grid_import_str.as_str(),
                grid_import_trend,
            ])
            .style(Style::default().fg(Color::White)),
            Row::new(vec![
                "  Grid Export",
                grid_export_str.as_str(),
                grid_export_trend,
            ])
            .style(Style::default().fg(Color::White)),
            Row::new(vec!["  Battery Power", battery_power_str.as_str(), "→"])
                .style(Style::default().fg(Color::White)),
        ],
        &[
            Constraint::Percentage(40),
            Constraint::Percentage(35),
            Constraint::Percentage(25),
        ],
    )
    .block(
        Block::default()
            .title(" REALTIME POWER ")
            .title_style(
                Style::default()
                    .fg(Color::Rgb(100, 200, 255))
                    .add_modifier(Modifier::BOLD),
            )
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Rgb(0, 180, 220))),
    )
    .column_spacing(2);

    f.render_widget(main_table, area);
}

fn render_system_status(f: &mut Frame, app: &AppState, area: Rect) {
    let latest = app.history.last();

    let battery_voltage = latest.map(|d| d.battery_voltage).unwrap_or(0.0);
    let battery_power = latest.map(|d| d.battery_power).unwrap_or(0.0);
    let battery_temp = latest.map(|d| d.battery_temp).unwrap_or(0.0);
    let battery_current = latest.map(|d| d.battery_current).unwrap_or(0.0);
    let load_current = latest.map(|d| d.load_current).unwrap_or(0.0);
    let load_val = latest.map(|d| d.load).unwrap_or(0.0);
    let load_power_factor = latest.map(|d| d.load_power_factor).unwrap_or(0.0);
    let grid_power_factor = latest.map(|d| d.grid_power_factor).unwrap_or(0.0);
    let inverter_temp = latest.map(|d| d.inverter_temp).unwrap_or(0.0);
    let inverter_voltage = latest.map(|d| d.inverter_voltage).unwrap_or(0.0);
    let inverter_status = latest
        .map(|d| d.inverter_status.clone())
        .unwrap_or_else(|| "Unknown".to_string());
    let grid_voltage = latest.map(|d| d.grid_voltage).unwrap_or(0.0);
    let inverter_freq = latest.map(|d| d.inverter_frequency).unwrap_or(0.0);
    let dc_trans_temp = latest.map(|d| d.dc_transformer_temp).unwrap_or(0.0);
    let radiator_temp = latest.map(|d| d.radiator_temp).unwrap_or(0.0);

    let battery_soc_value = battery_soc(latest);

    // Calculate remaining battery capacity if battery_capacity_kwh is configured
    let battery_soc_str = if let Some(capacity_kwh) = app.config.home_assistant.battery_capacity_kwh
    {
        let remaining_kwh = (battery_soc_value / 100.0) * capacity_kwh;
        format!("{:.1}% ({:.2} kWh)", battery_soc_value, remaining_kwh)
    } else {
        format!("{:.1}%", battery_soc_value)
    };

    // Check if battery is floating
    let is_floating = is_battery_floating(app, battery_voltage, battery_power);

    let battery_voltage_str = if is_floating {
        format!("{:.1} V 🟢FLOAT", battery_voltage)
    } else {
        format!("{:.1} V", battery_voltage)
    };

    let battery_current_str = format!("{:.1} A", battery_current);
    let load_current_str = format!("{:.1} A", load_current);
    let load_power_str = format!("{:.2} kW", load_val / 1000.0);
    let load_power_factor_str = format!("{:.2}", load_power_factor);
    let grid_power_factor_str = format!("{:.2}", grid_power_factor);
    let battery_temp_str = format!("{:.1} °C", battery_temp);
    let inverter_temp_str = format!("{:.1} °C", inverter_temp);
    let inverter_voltage_str = format!("{:.1} V", inverter_voltage);
    let grid_voltage_str = format!("{:.1} V", grid_voltage);
    let inverter_freq_str = format!("{:.1} Hz", inverter_freq);
    let dc_trans_temp_str = format!("{:.1} °C", dc_trans_temp);
    let radiator_temp_str = format!("{:.1} °C", radiator_temp);

    // Helper function to get temperature color (white for normal, only warn on high temps)
    let get_temp_color = |temp: f64| -> Color {
        if temp >= 35.0 {
            Color::Rgb(255, 165, 0) // Orange for high temp
        } else {
            Color::White // White for normal temps
        }
    };

    // Helper function to get AC voltage color (white for normal, only warn on low voltage)
    let get_voltage_color = |voltage: f64| -> Color {
        if voltage < 210.0 {
            Color::Red // Red for critical low
        } else if voltage < 220.0 {
            Color::Rgb(255, 165, 0) // Orange for low voltage
        } else {
            Color::White // White for normal voltage
        }
    };

    // Helper function to get current color (white for normal, warn on high current)
    let get_current_color = |current: f64| -> Color {
        if current >= 100.0 {
            Color::Red // Red for very high current
        } else if current >= 80.0 {
            Color::Rgb(255, 165, 0) // Orange for high current
        } else {
            Color::White // White for normal current
        }
    };

    // Split area into 2x2 grid
    let vertical_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    let top_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(vertical_chunks[0]);

    let bottom_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(vertical_chunks[1]);

    // Top-left: BATTERY
    let battery_table = Table::new(
        vec![
            Row::new(vec!["  SOC", battery_soc_str.as_str()])
                .style(Style::default().fg(Color::White)),
            Row::new(vec!["  Voltage", battery_voltage_str.as_str()])
                .style(Style::default().fg(Color::White)),
            Row::new(vec!["  Current", battery_current_str.as_str()])
                .style(Style::default().fg(Color::White)),
            Row::new(vec![
                Span::styled("  Temperature", Style::default().fg(Color::White)),
                Span::styled(
                    &battery_temp_str,
                    Style::default().fg(get_temp_color(battery_temp)),
                ),
            ]),
        ],
        &[Constraint::Length(14), Constraint::Min(0)],
    )
    .block(
        Block::default()
            .title(" BATTERY ")
            .title_style(
                Style::default()
                    .fg(Color::Rgb(100, 200, 255))
                    .add_modifier(Modifier::BOLD),
            )
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Rgb(100, 200, 255))),
    )
    .column_spacing(3);

    // Top-right: LOAD
    let load_table = Table::new(
        vec![
            Row::new(vec!["  Power", load_power_str.as_str()])
                .style(Style::default().fg(Color::White)),
            Row::new(vec![
                Span::styled("  Voltage", Style::default().fg(Color::White)),
                Span::styled(
                    &inverter_voltage_str,
                    Style::default().fg(get_voltage_color(inverter_voltage)),
                ),
            ]),
            Row::new(vec![
                Span::styled("  Current", Style::default().fg(Color::White)),
                Span::styled(
                    &load_current_str,
                    Style::default().fg(get_current_color(load_current)),
                ),
            ]),
            Row::new(vec!["  Power Factor", load_power_factor_str.as_str()])
                .style(Style::default().fg(Color::White)),
        ],
        &[Constraint::Length(15), Constraint::Min(0)],
    )
    .block(
        Block::default()
            .title(" LOAD ")
            .title_style(
                Style::default()
                    .fg(Color::Rgb(138, 161, 255))
                    .add_modifier(Modifier::BOLD),
            )
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Rgb(138, 161, 255))),
    )
    .column_spacing(3);

    // Bottom-left: GRID
    let grid_table = Table::new(
        vec![
            Row::new(vec![
                Span::styled("  Voltage", Style::default().fg(Color::White)),
                Span::styled(
                    &grid_voltage_str,
                    Style::default().fg(get_voltage_color(grid_voltage)),
                ),
            ]),
            Row::new(vec!["  Frequency", inverter_freq_str.as_str()])
                .style(Style::default().fg(Color::White)),
            Row::new(vec!["  Power Factor", grid_power_factor_str.as_str()])
                .style(Style::default().fg(Color::White)),
        ],
        &[Constraint::Length(15), Constraint::Min(0)],
    )
    .block(
        Block::default()
            .title(" GRID ")
            .title_style(
                Style::default()
                    .fg(Color::Rgb(100, 255, 150))
                    .add_modifier(Modifier::BOLD),
            )
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Rgb(100, 255, 150))),
    )
    .column_spacing(3);

    // Bottom-right: INVERTER
    let inverter_table = Table::new(
        vec![
            Row::new(vec!["  Status", inverter_status.as_str()])
                .style(Style::default().fg(Color::White)),
            Row::new(vec![
                Span::styled("  Temperature", Style::default().fg(Color::White)),
                Span::styled(
                    &inverter_temp_str,
                    Style::default().fg(get_temp_color(inverter_temp)),
                ),
            ]),
            Row::new(vec![
                Span::styled("  AC Temp", Style::default().fg(Color::White)),
                Span::styled(
                    &radiator_temp_str,
                    Style::default().fg(get_temp_color(radiator_temp)),
                ),
            ]),
            Row::new(vec![
                Span::styled("  DC Combiner", Style::default().fg(Color::White)),
                Span::styled(
                    &dc_trans_temp_str,
                    Style::default().fg(get_temp_color(dc_trans_temp)),
                ),
            ]),
        ],
        &[Constraint::Length(14), Constraint::Min(0)],
    )
    .block(
        Block::default()
            .title(" INVERTER ")
            .title_style(
                Style::default()
                    .fg(Color::Rgb(255, 200, 100))
                    .add_modifier(Modifier::BOLD),
            )
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Rgb(255, 200, 100))),
    )
    .column_spacing(3);

    // Render all 4 tables
    f.render_widget(battery_table, top_chunks[0]);
    f.render_widget(load_table, top_chunks[1]);
    f.render_widget(grid_table, bottom_chunks[0]);
    f.render_widget(inverter_table, bottom_chunks[1]);
}

fn render_daily_energy_compact(f: &mut Frame, app: &AppState, area: Rect) {
    let latest = app.history.last();

    let day_pv = latest.map(|d| d.day_pv_energy).unwrap_or(0.0);
    let day_load = latest.map(|d| d.day_load_energy).unwrap_or(0.0);
    let day_bat_charge = latest.map(|d| d.day_battery_charge).unwrap_or(0.0);
    let day_bat_discharge = latest.map(|d| d.day_battery_discharge).unwrap_or(0.0);
    let day_grid_import = latest.map(|d| d.day_grid_import).unwrap_or(0.0);
    let day_grid_export = latest.map(|d| d.day_grid_export).unwrap_or(0.0);

    // Configurable scale from config, default to 100 kWh
    let max_scale = app.config.home_assistant.max_daily_energy.unwrap_or(100.0);

    let block = Block::default()
        .title(" DAILY ENERGY TOTALS ")
        .title_style(
            Style::default()
                .fg(Color::Rgb(255, 150, 255))
                .add_modifier(Modifier::BOLD),
        )
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Rgb(200, 50, 200)));

    let inner = block.inner(area);
    f.render_widget(block, area);

    // Render each bar
    let bar_height = 1;
    let items = [
        ("PV Generation", day_pv, Color::Rgb(255, 215, 0)),
        ("Load Consumption", day_load, Color::Rgb(138, 161, 255)),
        ("Battery Charge", day_bat_charge, Color::Rgb(100, 255, 100)),
        (
            "Battery Discharge",
            day_bat_discharge,
            Color::Rgb(255, 165, 0),
        ),
        ("Grid Import", day_grid_import, Color::Rgb(255, 100, 100)),
        ("Grid Export", day_grid_export, Color::Rgb(100, 200, 255)),
    ];

    for (idx, (label, value, color)) in items.iter().enumerate() {
        let y = inner.y + idx as u16;
        if y >= inner.y + inner.height {
            break;
        }

        // Calculate bar width (leave space for label and value)
        let label_width = 20; // Width for label
        let value_width = 13; // Width for value display
        let available_width = inner.width.saturating_sub(label_width + value_width + 2);

        // Scale bar based on configurable max_daily_energy range
        let normalized_value = (value / max_scale).min(1.0); // Cap at 100%
        let filled_bar_width = (normalized_value * available_width as f64) as u16;
        let value_str = format!("{:>6.2} kWh", value);

        // Render label (right-aligned)
        let label_area = Rect::new(inner.x, y, label_width, bar_height);
        let label_text =
            Paragraph::new(format!("{:>18} |", label)).style(Style::default().fg(Color::White));
        f.render_widget(label_text, label_area);

        // Render value
        let value_area = Rect::new(inner.x + label_width + 1, y, value_width, bar_height);
        let value_text =
            Paragraph::new(value_str.to_string()).style(Style::default().fg(Color::White));
        f.render_widget(value_text, value_area);

        // Render background bar (full scale, low opacity - using darker version of color)
        if available_width > 0 {
            let bg_bar_area = Rect::new(
                inner.x + label_width + value_width + 2,
                y,
                available_width,
                bar_height,
            );

            // Create dimmed version of the color (reduce RGB values by ~75% for low opacity effect)
            let dim_color = match color {
                Color::Rgb(r, g, b) => Color::Rgb(r / 4, g / 4, b / 4),
                _ => Color::Rgb(30, 30, 30),
            };

            let bg_bar = Block::default().style(Style::default().bg(dim_color));
            f.render_widget(bg_bar, bg_bar_area);
        }

        // Render filled bar with gradient (actual value, full brightness)
        if filled_bar_width > 0 {
            let bar_x_start = inner.x + label_width + value_width + 2;

            // Create gradient by dividing bar into segments
            let gradient_steps = filled_bar_width.min(10); // Max 10 steps for performance
            let step_width = filled_bar_width as f64 / gradient_steps as f64;

            for step in 0..gradient_steps {
                let segment_x = bar_x_start + (step as f64 * step_width) as u16;
                let next_segment_x = bar_x_start + ((step + 1) as f64 * step_width) as u16;
                let segment_width = next_segment_x.saturating_sub(segment_x);

                if segment_width == 0 {
                    continue;
                }

                // Calculate gradient color based on position (0.0 to 1.0)
                let position = step as f64 / gradient_steps as f64;
                let gradient_color = apply_gradient(*color, position);

                let segment_area = Rect::new(segment_x, y, segment_width, bar_height);
                let segment = Block::default().style(Style::default().bg(gradient_color));
                f.render_widget(segment, segment_area);
            }
        }
    }
}

fn render_reserved_space(f: &mut Frame, app: &AppState, area: Rect) {
    let latest = app.history.last();

    let total_pv_gen = latest.map(|d| d.total_pv_generation).unwrap_or(0.0);
    let all_time_energy_peak = latest.map(|d| d.all_time_energy_usage_peak).unwrap_or(0.0);
    let all_time_energy_peak_date = latest
        .map(|d| d.all_time_energy_usage_peak_date.as_str())
        .unwrap_or("N/A");
    let all_time_load_peak = latest.map(|d| d.all_time_load_peak).unwrap_or(0.0);
    let all_time_pv_power_peak = latest.map(|d| d.all_time_pv_power_peak).unwrap_or(0.0);
    let all_time_pv_power_peak_date = latest
        .map(|d| d.all_time_pv_power_peak_date.as_str())
        .unwrap_or("N/A");
    let all_time_pv_yield_peak = latest.map(|d| d.all_time_pv_yield_peak).unwrap_or(0.0);
    let all_time_pv_yield_peak_date = latest
        .map(|d| d.all_time_pv_yield_peak_date.as_str())
        .unwrap_or("N/A");
    let daily_pv_power_peak = latest.map(|d| d.daily_pv_power_peak).unwrap_or(0.0);
    let daily_pv_power_peak_date = latest
        .map(|d| d.daily_pv_power_peak_date.as_str())
        .unwrap_or("N/A");
    let load_ratio = latest.map(|d| d.load_ratio).unwrap_or(0.0);
    let night_consume = latest.map(|d| d.night_consume).unwrap_or(0.0);
    let pv_forecast_remain = latest.map(|d| d.pv_forecast_remain).unwrap_or(0.0);
    let pv_forecast_today = latest.map(|d| d.pv_forecast_today).unwrap_or(0.0);
    let load_energy_yesterday = latest.map(|d| d.load_energy_yesterday).unwrap_or(0.0);
    let load_energy_total = latest.map(|d| d.load_energy_total).unwrap_or(0.0);
    let day_consume = latest.map(|d| d.day_consume).unwrap_or(0.0);

    // Format all strings beforehand to avoid lifetime issues
    let total_pv_str = format!("{:.2} kWh", total_pv_gen);
    let energy_peak_str = format!(
        "{:.2} kWh ({})",
        all_time_energy_peak, all_time_energy_peak_date
    );
    let load_peak_str = format!("{:.2} kW", all_time_load_peak);
    let pv_power_peak_str = format!(
        "{:.2} kW ({})",
        all_time_pv_power_peak, all_time_pv_power_peak_date
    );
    let pv_yield_peak_str = format!(
        "{:.2} kWh ({})",
        all_time_pv_yield_peak, all_time_pv_yield_peak_date
    );
    let daily_pv_peak_str = format!(
        "{:.2} kW ({})",
        daily_pv_power_peak, daily_pv_power_peak_date
    );
    let load_ratio_str = format!("{:.1}%", load_ratio);

    // Calculate day/night consumption with ratios
    let total_day_night = day_consume + night_consume;
    let day_ratio = if total_day_night > 0.0 {
        (day_consume / total_day_night) * 100.0
    } else {
        0.0
    };
    let night_ratio = if total_day_night > 0.0 {
        (night_consume / total_day_night) * 100.0
    } else {
        0.0
    };
    let day_night_consume_str = format!(
        "{:.2}/{:.2} kWh ({:.1}/{:.1}%)",
        day_consume, night_consume, day_ratio, night_ratio
    );

    let pv_forecast_today_str = format!("{:.2} kWh", pv_forecast_today);
    let pv_forecast_remain_str = format!("{:.2} kWh", pv_forecast_remain);
    let load_energy_yesterday_str = format!("{:.2} kWh", load_energy_yesterday);
    let load_energy_total_str = format!("{:.2} kWh", load_energy_total);

    // Extract dates from history (before creating table to avoid borrowing issues)
    let pv_power_peak_date = app
        .history
        .last()
        .map(|d| d.all_time_pv_power_peak_date.clone())
        .unwrap_or_else(|| String::from("-"));
    let pv_yield_peak_date = app
        .history
        .last()
        .map(|d| d.all_time_pv_yield_peak_date.clone())
        .unwrap_or_else(|| String::from("-"));
    let daily_pv_power_peak_date = app
        .history
        .last()
        .map(|d| d.daily_pv_power_peak_date.clone())
        .unwrap_or_else(|| String::from("-"));
    let energy_peak_date = app
        .history
        .last()
        .map(|d| d.all_time_energy_usage_peak_date.clone())
        .unwrap_or_else(|| String::from("-"));

    let table = Table::new(
        vec![
            // Category 1: PV / Solar Production
            Row::new(vec![
                "─────────────────────────────────────────".to_string(),
                "─────────────────────────────────────────".to_string(),
            ])
            .style(Style::default().fg(Color::Rgb(100, 100, 100))),
            Row::new(vec!["  Total PV Generated", total_pv_str.as_str()])
                .style(Style::default().fg(Color::White)),
            Row::new(vec!["  All-Time PV Power Peak", pv_power_peak_str.as_str()])
                .style(Style::default().fg(Color::White)),
            Row::new(vec!["  PV Power Peak Date", pv_power_peak_date.as_str()])
                .style(Style::default().fg(Color::Rgb(200, 200, 200))),
            Row::new(vec!["  All-Time PV Yield Peak", pv_yield_peak_str.as_str()])
                .style(Style::default().fg(Color::White)),
            Row::new(vec!["  PV Yield Peak Date", pv_yield_peak_date.as_str()])
                .style(Style::default().fg(Color::Rgb(200, 200, 200))),
            Row::new(vec!["  Today PV Power Peak", daily_pv_peak_str.as_str()])
                .style(Style::default().fg(Color::White)),
            Row::new(vec![
                "  Today PV Power Peak Date",
                daily_pv_power_peak_date.as_str(),
            ])
            .style(Style::default().fg(Color::Rgb(200, 200, 200))),
            // Category 2: Load Consumption
            Row::new(vec![
                "─────────────────────────────────────────".to_string(),
                "─────────────────────────────────────────".to_string(),
            ])
            .style(Style::default().fg(Color::Rgb(100, 100, 100))),
            Row::new(vec!["  All-Time Load Peak", load_peak_str.as_str()])
                .style(Style::default().fg(Color::White)),
            Row::new(vec!["  Load Ratio", load_ratio_str.as_str()])
                .style(Style::default().fg(Color::White)),
            Row::new(vec!["  Yesterday Load", load_energy_yesterday_str.as_str()])
                .style(Style::default().fg(Color::White)),
            Row::new(vec![
                "  Total Load Consumed",
                load_energy_total_str.as_str(),
            ])
            .style(Style::default().fg(Color::White)),
            // Category 3: Energy Usage Peak
            Row::new(vec![
                "─────────────────────────────────────────".to_string(),
                "─────────────────────────────────────────".to_string(),
            ])
            .style(Style::default().fg(Color::Rgb(100, 100, 100))),
            Row::new(vec!["  All-Time Energy Peak", energy_peak_str.as_str()])
                .style(Style::default().fg(Color::White)),
            Row::new(vec!["  Energy Peak Date", energy_peak_date.as_str()])
                .style(Style::default().fg(Color::Rgb(200, 200, 200))),
            // Category 4: Consumption Analysis
            Row::new(vec![
                "─────────────────────────────────────────".to_string(),
                "─────────────────────────────────────────".to_string(),
            ])
            .style(Style::default().fg(Color::Rgb(100, 100, 100))),
            Row::new(vec![
                "  Day/Night Consumption",
                day_night_consume_str.as_str(),
            ])
            .style(Style::default().fg(Color::White)),
            // Category 5: PV Forecast
            Row::new(vec![
                "─────────────────────────────────────────".to_string(),
                "─────────────────────────────────────────".to_string(),
            ])
            .style(Style::default().fg(Color::Rgb(100, 100, 100))),
            Row::new(vec!["  PV Forecast Today", pv_forecast_today_str.as_str()])
                .style(Style::default().fg(Color::White)),
            Row::new(vec![
                "  PV Forecast Remain",
                pv_forecast_remain_str.as_str(),
            ])
            .style(Style::default().fg(Color::White)),
        ],
        &[Constraint::Percentage(50), Constraint::Percentage(50)],
    )
    .block(
        Block::default()
            .title("  SUMMARY & STATISTICS  ")
            .title_style(
                Style::default()
                    .fg(Color::Rgb(180, 180, 255))
                    .add_modifier(Modifier::BOLD),
            )
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Rgb(120, 120, 200)))
            .padding(Padding::horizontal(1)),
    )
    .column_spacing(2);

    f.render_widget(table, area);
}

fn solar_val(latest: Option<&crate::state::PowerData>) -> f64 {
    latest.map(|d| d.solar).unwrap_or(0.0)
}

fn load_val(latest: Option<&crate::state::PowerData>) -> f64 {
    latest.map(|d| d.load).unwrap_or(0.0)
}

fn battery_soc(latest: Option<&crate::state::PowerData>) -> f64 {
    latest.map(|d| d.battery_soc).unwrap_or(0.0)
}

fn is_battery_floating(app: &AppState, battery_voltage: f64, battery_power: f64) -> bool {
    let float_voltage_threshold = app
        .config
        .home_assistant
        .battery_float_voltage
        .unwrap_or(54.0);
    let voltage_tolerance = 0.5;

    (battery_voltage >= float_voltage_threshold - voltage_tolerance)
        && (battery_voltage <= float_voltage_threshold + voltage_tolerance)
        && battery_power.abs() < 50.0
}

fn get_power_trend<F>(history: &[crate::state::PowerData], get_value: F) -> &'static str
where
    F: Fn(&crate::state::PowerData) -> f64,
{
    if history.len() < 2 {
        return "—";
    }

    let current = get_value(&history[history.len() - 1]);
    let previous = get_value(&history[history.len() - 2]);

    if current > previous * 1.05 {
        "↑ Rising"
    } else if current < previous * 0.95 {
        "↓ Falling"
    } else {
        "→ Stable"
    }
}

fn format_battery_power(power: f64, is_floating: bool) -> String {
    if power > 10.0 {
        format!("🔌 {:.2} kW (Charging)", power / 1000.0)
    } else if power < -10.0 {
        format!("⚡ {:.2} kW (Discharging)", (-power) / 1000.0)
    } else if is_floating {
        "Idle (Floating)".to_string()
    } else {
        "Idle".to_string()
    }
}

fn get_gradient_color(ratio: f64) -> Color {
    // Rainbow gradient: Red -> Orange -> Yellow -> Green -> Cyan -> Light Blue (50 steps, no magenta)
    get_rainbow_gradient(ratio)
}

fn get_rainbow_gradient(ratio: f64) -> Color {
    // Darker, saturated colors for better white text readability
    // All colors scaled to ~50% brightness
    let colors = [
        Color::Rgb(180, 0, 0),   // 0 - Red (darker)
        Color::Rgb(180, 10, 0),  // 1
        Color::Rgb(180, 20, 0),  // 2
        Color::Rgb(180, 30, 0),  // 3
        Color::Rgb(180, 40, 0),  // 4
        Color::Rgb(180, 50, 0),  // 5 - Red-Orange
        Color::Rgb(180, 60, 0),  // 6
        Color::Rgb(180, 70, 0),  // 7
        Color::Rgb(180, 80, 0),  // 8
        Color::Rgb(180, 90, 0),  // 9
        Color::Rgb(180, 100, 0), // 10 - Orange
        Color::Rgb(180, 110, 0), // 11
        Color::Rgb(180, 120, 0), // 12
        Color::Rgb(180, 130, 0), // 13
        Color::Rgb(180, 140, 0), // 14
        Color::Rgb(180, 150, 0), // 15 - Orange-Yellow
        Color::Rgb(180, 155, 0), // 16
        Color::Rgb(180, 160, 0), // 17
        Color::Rgb(180, 165, 0), // 18
        Color::Rgb(180, 170, 0), // 19
        Color::Rgb(180, 175, 0), // 20 - Yellow
        Color::Rgb(160, 175, 0), // 21
        Color::Rgb(140, 175, 0), // 22
        Color::Rgb(120, 175, 0), // 23
        Color::Rgb(100, 175, 0), // 24
        Color::Rgb(80, 175, 0),  // 25 - Yellow-Green
        Color::Rgb(60, 175, 0),  // 26
        Color::Rgb(40, 175, 0),  // 27
        Color::Rgb(20, 175, 0),  // 28
        Color::Rgb(0, 175, 0),   // 30 - Green
        Color::Rgb(0, 175, 20),  // 31
        Color::Rgb(0, 175, 40),  // 32
        Color::Rgb(0, 175, 60),  // 33
        Color::Rgb(0, 175, 80),  // 34
        Color::Rgb(0, 175, 100), // 35 - Green-Cyan
        Color::Rgb(0, 175, 120), // 36
        Color::Rgb(0, 175, 140), // 37
        Color::Rgb(0, 175, 160), // 38 - Cyan
        Color::Rgb(0, 160, 175), // 39
        Color::Rgb(0, 145, 175), // 40
        Color::Rgb(0, 130, 175), // 41
        Color::Rgb(0, 115, 175), // 42 - Cyan-Blue
        Color::Rgb(5, 105, 165), // 43
        Color::Rgb(10, 95, 155), // 44
        Color::Rgb(15, 85, 145), // 45
        Color::Rgb(20, 75, 135), // 46 - Medium Blue (darker)
        Color::Rgb(25, 80, 140), // 47
        Color::Rgb(30, 85, 145), // 48
        Color::Rgb(35, 90, 150), // 49
    ];

    // Smooth interpolation
    let pos = ratio * (colors.len() - 1) as f64;
    let idx = pos.floor() as usize;
    let next_idx = (idx + 1).min(colors.len() - 1);
    let t = pos - idx as f64;

    if t < 0.001 {
        colors[idx]
    } else {
        let Color::Rgb(r1, g1, b1) = colors[idx] else {
            unreachable!()
        };
        let Color::Rgb(r2, g2, b2) = colors[next_idx] else {
            unreachable!()
        };

        Color::Rgb(
            (r1 as f64 + (r2 as f64 - r1 as f64) * t) as u8,
            (g1 as f64 + (g2 as f64 - g1 as f64) * t) as u8,
            (b1 as f64 + (b2 as f64 - b1 as f64) * t) as u8,
        )
    }
}

fn get_blue_gradient_color(ratio: f64) -> Color {
    // Rainbow gradient for load gauge
    get_rainbow_gradient(ratio)
}

fn get_soc_gradient_color(ratio: f64) -> Color {
    get_rainbow_gradient(ratio)
}
fn get_solar_text_color(_ratio: f64) -> Color {
    // Always use white text for solar gauge for better aesthetics
    Color::Rgb(255, 255, 255)
}

fn get_load_text_color(_ratio: f64) -> Color {
    // Always use white text for LOAD gauge for better visibility
    Color::Rgb(255, 255, 255)
}

fn get_soc_text_color(_ratio: f64) -> Color {
    // Always use white text for SOC gauge for better visibility
    Color::Rgb(255, 255, 255)
}

fn apply_gradient(base_color: Color, position: f64) -> Color {
    // Apply gradient from darker (left) to brighter (right)
    // position ranges from 0.0 (darkest) to 1.0 (brightest)
    if let Color::Rgb(r, g, b) = base_color {
        // Start at 40% brightness, end at 100% brightness
        let brightness_factor = 0.4 + (0.6 * position);
        Color::Rgb(
            (r as f64 * brightness_factor) as u8,
            (g as f64 * brightness_factor) as u8,
            (b as f64 * brightness_factor) as u8,
        )
    } else {
        base_color
    }
}

fn render_error_popup(f: &mut Frame, app: &AppState, area: Rect) {
    if let Some(error) = &app.error {
        // Calculate popup size (centered, 60% width, auto height)
        let popup_width = (area.width as f32 * 0.6).min(80.0) as u16;
        let popup_height = 10;

        let popup_x = (area.width.saturating_sub(popup_width)) / 2;
        let popup_y = (area.height.saturating_sub(popup_height)) / 2;

        let popup_area = Rect {
            x: popup_x,
            y: popup_y,
            width: popup_width,
            height: popup_height,
        };

        // Clear the area behind the popup
        f.render_widget(Clear, popup_area);

        // Error message - wrap text if too long
        let error_msg = error.to_string();
        let max_line_width = (popup_width as usize).saturating_sub(6);
        let wrapped_lines = wrap_text(&error_msg, max_line_width);

        let mut text_lines = vec![
            Line::from(Span::styled(
                "⚠ CONNECTION ERROR",
                Style::default()
                    .fg(Color::Rgb(255, 100, 100))
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Auto-reconnecting...",
                Style::default()
                    .fg(Color::Rgb(200, 200, 200))
                    .add_modifier(Modifier::ITALIC),
            )),
            Line::from(""),
        ];

        for line in wrapped_lines {
            text_lines.push(Line::from(Span::styled(
                line,
                Style::default().fg(Color::White),
            )));
        }

        text_lines.push(Line::from(""));
        text_lines.push(Line::from(Span::styled(
            "Press any key to dismiss or wait for reconnection",
            Style::default()
                .fg(Color::Rgb(150, 150, 150))
                .add_modifier(Modifier::ITALIC),
        )));

        let popup = Paragraph::new(text_lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(Color::Rgb(255, 50, 50)))
                    .style(Style::default().bg(Color::Rgb(30, 10, 10))),
            )
            .alignment(Alignment::Center);

        f.render_widget(popup, popup_area);
    }
}

fn wrap_text(text: &str, max_width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current_line = String::new();

    for word in text.split_whitespace() {
        if current_line.is_empty() {
            current_line = word.to_string();
        } else if current_line.len() + word.len() < max_width {
            current_line.push(' ');
            current_line.push_str(word);
        } else {
            lines.push(current_line.clone());
            current_line = word.to_string();
        }
    }

    if !current_line.is_empty() {
        lines.push(current_line);
    }

    lines
}

fn render_terminal_too_small(f: &mut Frame, area: Rect, min_width: u16, min_height: u16) {
    let msg = "Terminal too small!".to_string();
    let min_size = format!("Minimum size: {}x{}", min_width, min_height);
    let current_size = format!("Current size: {}x{}", area.width, area.height);
    let resize_msg = "Resize terminal to continue...".to_string();

    let lines = vec![
        Line::from(msg),
        Line::from(""),
        Line::from(min_size),
        Line::from(current_size),
        Line::from(""),
        Line::from(resize_msg),
    ];

    let text_height = lines.len() as u16 + 2;
    let popup_width = 40.min(area.width);

    let popup_area = Rect {
        x: (area.width.saturating_sub(popup_width)) / 2,
        y: area.height.saturating_sub(text_height) / 2,
        width: popup_width,
        height: text_height,
    };

    let paragraph = Paragraph::new(lines).alignment(Alignment::Center).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Red))
            .style(Style::default().bg(Color::Black)),
    );

    f.render_widget(Clear, popup_area);
    f.render_widget(paragraph, popup_area);
}
