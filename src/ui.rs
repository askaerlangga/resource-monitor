use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, BorderType, Paragraph, Row, Table, Tabs, Chart, Dataset, GraphType, Axis, Clear},
    symbols,
    Frame,
};
use crate::app::App;

const COLOR_MAUVE: Color = Color::Rgb(198, 160, 246);
const COLOR_PEACH: Color = Color::Rgb(245, 169, 127);
const COLOR_YELLOW: Color = Color::Rgb(238, 212, 159);
const COLOR_GREEN: Color = Color::Rgb(166, 218, 149);
const COLOR_SKY: Color = Color::Rgb(145, 215, 227);
const COLOR_SAPPHIRE: Color = Color::Rgb(125, 196, 228);
const COLOR_LAVENDER: Color = Color::Rgb(180, 190, 254);
const COLOR_TEXT: Color = Color::Rgb(202, 211, 245);
const COLOR_SUBTEXT0: Color = Color::Rgb(165, 173, 206);
const COLOR_SURFACE1: Color = Color::Rgb(91, 96, 120);
const COLOR_RED: Color = Color::Rgb(237, 135, 150);
const COLOR_TEAL: Color = Color::Rgb(139, 213, 202);
const COLOR_ORANGE: Color = Color::Rgb(245, 169, 100);

fn resolve(color: Color, no_truecolor: bool) -> Color {
    if !no_truecolor { return color; }
    match color {
        Color::Rgb(198, 160, 246) => Color::Magenta,
        Color::Rgb(245, 169, 127) => Color::Yellow,
        Color::Rgb(245, 169, 100) => Color::Yellow,
        Color::Rgb(238, 212, 159) => Color::Yellow,
        Color::Rgb(166, 218, 149) => Color::Green,
        Color::Rgb(145, 215, 227) => Color::Cyan,
        Color::Rgb(125, 196, 228) => Color::Cyan,
        Color::Rgb(139, 213, 202) => Color::Cyan,
        Color::Rgb(180, 190, 254) => Color::White,
        Color::Rgb(202, 211, 245) => Color::White,
        Color::Rgb(165, 173, 206) => Color::Gray,
        Color::Rgb(91, 96, 120) => Color::DarkGray,
        Color::Rgb(237, 135, 150) => Color::Red,
        _ => color,
    }
}

fn create_block_colored(title: &str, color: Color, no_truecolor: bool) -> Block<'static> {
    Block::default()
        .title(title.to_string())
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .style(Style::default().fg(resolve(color, no_truecolor)))
}

fn create_block(title: &str, no_truecolor: bool) -> Block<'static> {
    create_block_colored(title, COLOR_TEXT, no_truecolor)
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

fn get_chart_data<T: Into<f64> + Copy>(history: &std::collections::VecDeque<T>, area_width: u16) -> Vec<(f64, f64)> {
    let width = if area_width > 2 { (area_width as usize - 2) * 2 } else { area_width as usize * 2 };
    let count = history.len().min(width);
    let pad = width.saturating_sub(count);

    let mut result = Vec::with_capacity(width);
    for i in 0..pad {
        result.push((i as f64, 0.0));
    }

    let iter = history.iter().rev().take(count).collect::<Vec<_>>();
    for (i, &val) in iter.into_iter().rev().enumerate() {
        result.push(((pad + i) as f64, val.into()));
    }
    result
}

fn build_chart<'a>(title: String, data: &'a [(f64, f64)], color: Color, max_y: f64, no_truecolor: bool) -> Chart<'a> {
    let max_x = if data.is_empty() { 100.0 } else { data.last().unwrap().0 };

    let dataset = Dataset::default()
        .marker(symbols::Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Style::default().fg(resolve(color, no_truecolor)))
        .data(data);

    Chart::new(vec![dataset])
        .block(create_block(&title, no_truecolor))
        .x_axis(Axis::default().bounds([0.0, max_x]).style(Style::default().fg(Color::Reset)))
        .y_axis(Axis::default().bounds([0.0, max_y]).style(Style::default().fg(Color::Reset)))
}

fn build_chart_alert<'a>(title: String, data: &'a [(f64, f64)], color: Color, max_y: f64, alert: bool, no_truecolor: bool) -> Chart<'a> {
    let max_x = if data.is_empty() { 100.0 } else { data.last().unwrap().0 };
    let border_color = if alert { COLOR_RED } else { COLOR_TEXT };

    let dataset = Dataset::default()
        .marker(symbols::Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Style::default().fg(resolve(color, no_truecolor)))
        .data(data);

    Chart::new(vec![dataset])
        .block(create_block_colored(&title, border_color, no_truecolor))
        .x_axis(Axis::default().bounds([0.0, max_x]).style(Style::default().fg(Color::Reset)))
        .y_axis(Axis::default().bounds([0.0, max_y]).style(Style::default().fg(Color::Reset)))
}

pub fn ui(f: &mut Frame, app: &mut App) {
    let tc = app.no_truecolor;

    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(f.area());

    let body_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(30),
            Constraint::Percentage(70),
        ])
        .split(main_chunks[0]);

    let usage_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Ratio(1, 4),
            Constraint::Ratio(1, 4),
            Constraint::Ratio(1, 4),
            Constraint::Ratio(1, 4),
        ])
        .split(body_chunks[0]);

    let cpu_avg = *app.cpu_history.back().unwrap_or(&0.0);
    let cpu_data = get_chart_data(&app.cpu_history, usage_chunks[0].width);
    f.render_widget(build_chart_alert(format!(" CPU: {:.1}% ", cpu_avg), &cpu_data, COLOR_SAPPHIRE, 100.0, app.alert_cpu, tc), usage_chunks[0]);

    let mem_avg = *app.mem_history.back().unwrap_or(&0.0);
    let mem_data = get_chart_data(&app.mem_history, usage_chunks[1].width);
    f.render_widget(build_chart_alert(format!(" MEM: {:.1}% ", mem_avg), &mem_data, COLOR_MAUVE, 100.0, app.alert_mem, tc), usage_chunks[1]);

    let net_in_avg = *app.network_in_history.back().unwrap_or(&0.0);
    let net_in_data = get_chart_data(&app.network_in_history, usage_chunks[2].width);
    let net_in_max = net_in_data.iter().map(|&(_, y)| y).max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap_or(1.0);
    let net_in_max = if net_in_max == 0.0 { 1.0 } else { net_in_max };
    f.render_widget(build_chart(format!(" Net IN: {:.2} KB/s ", net_in_avg), &net_in_data, COLOR_GREEN, net_in_max, tc), usage_chunks[2]);

    let disk_total_avg = *app.disk_total_history.back().unwrap_or(&0.0);
    let disk_total_data = get_chart_data(&app.disk_total_history, usage_chunks[3].width);
    let disk_max = disk_total_data.iter().map(|&(_, y)| y).max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap_or(1.0);
    let disk_max = if disk_max == 0.0 { 1.0 } else { disk_max };
    f.render_widget(build_chart(format!(" Disk I/O: {:.1} KB/s ", disk_total_avg), &disk_total_data, COLOR_ORANGE, disk_max, tc), usage_chunks[3]);

    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
        ])
        .split(body_chunks[1]);

    let titles = vec![" 1:Processes ", " 2:CPU ", " 3:Memory ", " 4:Network ", " 5:Disk "];
    f.render_widget(Tabs::new(titles)
        .block(create_block(" Views ", tc))
        .select(app.active_tab)
        .style(Style::default().fg(resolve(COLOR_SUBTEXT0, tc)))
        .highlight_style(Style::default().fg(resolve(COLOR_YELLOW, tc)).add_modifier(Modifier::BOLD)), right_chunks[0]);

    match app.active_tab {
        0 => {
            let sort_indicator = match app.sort_by {
                crate::app::SortBy::Cpu => "CPU",
                crate::app::SortBy::Memory => "Mem",
                crate::app::SortBy::DiskRead => "DiskR",
                crate::app::SortBy::DiskWrite => "DiskW",
            };

            let process_title = if !app.search_query.is_empty() {
                format!(" Processes [sort:{} | filter: \"{}\"] ({}/{}) ", sort_indicator, app.search_query, app.filtered_processes.len(), app.processes.len())
            } else {
                format!(" Processes [sort:{}] ", sort_indicator)
            };

            let title_color = if !app.search_query.is_empty() { COLOR_YELLOW } else { COLOR_TEXT };

            let process_header = Row::new(vec!["PID", "User", "Name", "CPU%", "Mem (MB)", "DiskR (KB/s)", "DiskW (KB/s)"])
                .style(Style::default().fg(resolve(COLOR_PEACH, tc)).add_modifier(Modifier::BOLD))
                .bottom_margin(1);
            let rows: Vec<Row> = app.filtered_processes.iter().enumerate().map(|(i, p)| {
                let row_color = if i % 2 == 0 { resolve(COLOR_TEXT, tc) } else { resolve(COLOR_SUBTEXT0, tc) };
                Row::new(vec![
                    p.pid.to_string(),
                    p.user.clone(),
                    p.name.clone(),
                    format!("{:.1}", p.cpu),
                    format!("{:.1}", p.mem as f64 / 1024.0 / 1024.0),
                    format!("{:.1}", p.disk_read as f64 / 1024.0),
                    format!("{:.1}", p.disk_write as f64 / 1024.0),
                ]).style(Style::default().fg(row_color))
            }).collect();
            let table = Table::new(rows, [
                Constraint::Percentage(8),
                Constraint::Percentage(10),
                Constraint::Percentage(28),
                Constraint::Percentage(9),
                Constraint::Percentage(15),
                Constraint::Percentage(15),
                Constraint::Percentage(15),
            ])
                .header(process_header)
                .block(Block::default()
                    .title(process_title)
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .style(Style::default().fg(resolve(title_color, tc))))
                .row_highlight_style(Style::default().bg(resolve(COLOR_SURFACE1, tc)).fg(resolve(COLOR_LAVENDER, tc)).add_modifier(Modifier::BOLD));
            f.render_stateful_widget(table, right_chunks[1], &mut app.table_state);
        }
        1 => {
            let cpu_detail_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
                .split(right_chunks[1]);

            let num_cpus = app.per_cpu_history.len();
            if num_cpus > 0 {
                let cols = if num_cpus > 8 { 2 } else { 1 };
                let rows = (num_cpus + cols - 1) / cols;
                let col_constraints = vec![Constraint::Percentage(100 / cols as u16); cols];
                let row_constraints = vec![Constraint::Ratio(1, rows as u32); rows];
                let grid_chunks = Layout::default().direction(Direction::Horizontal).constraints(col_constraints).split(cpu_detail_chunks[0]);
                for c in 0..cols {
                    let col_cells = Layout::default().direction(Direction::Vertical).constraints(row_constraints.clone()).split(grid_chunks[c]);
                    for r in 0..rows {
                        let cpu_idx = r * cols + c;
                        if cpu_idx < num_cpus {
                            let usage = *app.per_cpu_history[cpu_idx].back().unwrap_or(&0.0);
                            let data = get_chart_data(&app.per_cpu_history[cpu_idx], col_cells[r].width);
                            let chart = build_chart_alert(format!(" CPU {} - {:.1}% ", cpu_idx, usage), &data, COLOR_SAPPHIRE, 100.0, app.alert_cpu, tc);
                            f.render_widget(chart, col_cells[r]);
                        }
                    }
                }
            }

            if !app.sys.cpus().is_empty() {
                let first_cpu = &app.sys.cpus()[0];
                let brand = first_cpu.brand().trim();
                let days = app.uptime / 86400;
                let hours = (app.uptime % 86400) / 3600;
                let mins = (app.uptime % 3600) / 60;
                let secs = app.uptime % 60;
                let uptime_str = if days > 0 { format!("{}d {:02}h {:02}m {:02}s", days, hours, mins, secs) } else { format!("{:02}h {:02}m {:02}s", hours, mins, secs) };
                let cpu_temp_str = app.components.iter()
                    .find(|c| c.label().to_lowercase().contains("cpu") || c.label().to_lowercase().contains("package") || c.label().to_lowercase().contains("tctl"))
                    .and_then(|c| c.temperature())
                    .map(|t| format!("{:.1} °C", t))
                    .unwrap_or_else(|| "N/A".to_string());
                let specs_text = vec![
                    format!(" Brand:          {}", brand),
                    format!(" Vendor ID:      {}", first_cpu.vendor_id()),
                    format!(" Logical Cores:  {}", num_cpus),
                    format!(" L1/L2/L3 Cache: {} / {} / {}", app.l1_cache, app.l2_cache, app.l3_cache),
                    format!(" Max Frequency:  {}", app.max_cpu_freq_static),
                    format!(" Cur Frequency:  {} MHz (avg)", app.current_cpu_freq),
                    format!(" CPU Governor:   {}", app.cpu_governor),
                    format!(" Temperature:    {}", cpu_temp_str),
                    format!(" Load Average:   {:.2}  {:.2}  {:.2}  (1m 5m 15m)", app.load_avg.0, app.load_avg.1, app.load_avg.2),
                    format!(" Uptime:         {}", uptime_str),
                ];
                f.render_widget(Paragraph::new(specs_text.join("\n")).block(create_block(" CPU Specifications ", tc)).style(Style::default().fg(resolve(COLOR_LAVENDER, tc))), cpu_detail_chunks[1]);
            }
        }
        2 => {
            let mem_swap_chunks = Layout::default().direction(Direction::Vertical).constraints([Constraint::Percentage(40), Constraint::Percentage(40), Constraint::Percentage(20)]).split(right_chunks[1]);
            let total_mem_gb = app.sys.total_memory() as f64 / 1024.0 / 1024.0 / 1024.0;
            let available_mem_gb = app.sys.available_memory() as f64 / 1024.0 / 1024.0 / 1024.0;
            let used_mem_gb = total_mem_gb - available_mem_gb;
            let mem_avg = *app.mem_history.back().unwrap_or(&0.0);
            let mem_data = get_chart_data(&app.mem_history, mem_swap_chunks[0].width);
            f.render_widget(build_chart_alert(format!(" RAM: {:.2} GB / {:.2} GB - {:.1}% ", used_mem_gb, total_mem_gb, mem_avg), &mem_data, COLOR_MAUVE, 100.0, app.alert_mem, tc), mem_swap_chunks[0]);
            let total_swap_gb = app.sys.total_swap() as f64 / 1024.0 / 1024.0 / 1024.0;
            let used_swap_gb = app.sys.used_swap() as f64 / 1024.0 / 1024.0 / 1024.0;
            let swap_avg = *app.swap_history.back().unwrap_or(&0.0);
            let swap_data = get_chart_data(&app.swap_history, mem_swap_chunks[1].width);
            f.render_widget(build_chart(format!(" Swap: {:.2} GB / {:.2} GB - {:.1}% ", used_swap_gb, total_swap_gb, swap_avg), &swap_data, COLOR_PEACH, 100.0, tc), mem_swap_chunks[1]);
            let mem_specs_text = vec![format!(" Total Physical RAM: {:.2} GB", total_mem_gb), format!(" Total Configured Swap: {:.2} GB", total_swap_gb)];
            f.render_widget(Paragraph::new(mem_specs_text.join("\n")).block(create_block(" Memory Capacity ", tc)).style(Style::default().fg(resolve(COLOR_LAVENDER, tc))), mem_swap_chunks[2]);
        }
        3 => {
            let net_chunks = Layout::default().direction(Direction::Vertical).constraints([Constraint::Percentage(40), Constraint::Percentage(40), Constraint::Percentage(20)]).split(right_chunks[1]);
            let net_in_avg = *app.network_in_history.back().unwrap_or(&0.0);
            let net_in_data = get_chart_data(&app.network_in_history, net_chunks[0].width);
            let net_in_max = net_in_data.iter().map(|&(_, y)| y).max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap_or(1.0);
            let net_in_max = if net_in_max == 0.0 { 1.0 } else { net_in_max };
            f.render_widget(build_chart(format!(" Network IN: {:.2} KB/s ", net_in_avg), &net_in_data, COLOR_GREEN, net_in_max, tc), net_chunks[0]);
            let net_out_avg = *app.network_out_history.back().unwrap_or(&0.0);
            let net_out_data = get_chart_data(&app.network_out_history, net_chunks[1].width);
            let net_out_max = net_out_data.iter().map(|&(_, y)| y).max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap_or(1.0);
            let net_out_max = if net_out_max == 0.0 { 1.0 } else { net_out_max };
            f.render_widget(build_chart(format!(" Network OUT: {:.2} KB/s ", net_out_avg), &net_out_data, COLOR_SKY, net_out_max, tc), net_chunks[1]);
            let net_header = Row::new(vec!["IFace", "MAC Address", "IPv4", "IPv6"]).style(Style::default().fg(resolve(COLOR_PEACH, tc)).add_modifier(Modifier::BOLD)).bottom_margin(1);
            let mut net_rows = vec![];
            for (i, (interface_name, data)) in app.networks.iter().enumerate() {
                let mac_str = data.mac_address().to_string();
                let ip_networks = data.ip_networks();
                let mut ipv4s = vec![]; let mut ipv6s = vec![];
                for ip_net in ip_networks { let ip_str = ip_net.addr.to_string(); if ip_str.contains('.') { ipv4s.push(ip_str); } else if ip_str.contains(':') { ipv6s.push(ip_str); } }
                let ipv4_str = if ipv4s.is_empty() { "None".to_string() } else { ipv4s.join(", ") };
                let ipv6_str = if ipv6s.is_empty() { "None".to_string() } else { ipv6s.join(", ") };
                let row_color = if i % 2 == 0 { resolve(COLOR_TEXT, tc) } else { resolve(COLOR_SUBTEXT0, tc) };
                net_rows.push(Row::new(vec![interface_name.clone(), mac_str, ipv4_str, ipv6_str]).style(Style::default().fg(row_color)));
            }
            if net_rows.is_empty() { net_rows.push(Row::new(vec!["No interfaces found", "", "", ""]).style(Style::default().fg(resolve(COLOR_SUBTEXT0, tc)))); }
            let net_table = Table::new(net_rows, [Constraint::Percentage(15), Constraint::Percentage(20), Constraint::Percentage(25), Constraint::Percentage(40)])
                .header(net_header)
                .block(create_block(" Network Interfaces ", tc))
                .row_highlight_style(Style::default().bg(resolve(COLOR_SURFACE1, tc)).fg(resolve(COLOR_LAVENDER, tc)).add_modifier(Modifier::BOLD));
            f.render_widget(net_table, net_chunks[2]);
        }
        4 => {
            let disk_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(40), Constraint::Percentage(40), Constraint::Percentage(20)])
                .split(right_chunks[1]);

            let disk_read_avg = *app.disk_read_history.back().unwrap_or(&0.0);
            let disk_read_data = get_chart_data(&app.disk_read_history, disk_chunks[0].width);
            let disk_read_max = disk_read_data.iter().map(|&(_, y)| y).max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap_or(1.0);
            let disk_read_max = if disk_read_max == 0.0 { 1.0 } else { disk_read_max };
            f.render_widget(build_chart(format!(" Disk Read: {:.2} KB/s ", disk_read_avg), &disk_read_data, COLOR_ORANGE, disk_read_max, tc), disk_chunks[0]);

            let disk_write_avg = *app.disk_write_history.back().unwrap_or(&0.0);
            let disk_write_data = get_chart_data(&app.disk_write_history, disk_chunks[1].width);
            let disk_write_max = disk_write_data.iter().map(|&(_, y)| y).max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap_or(1.0);
            let disk_write_max = if disk_write_max == 0.0 { 1.0 } else { disk_write_max };
            f.render_widget(build_chart(format!(" Disk Write: {:.2} KB/s ", disk_write_avg), &disk_write_data, COLOR_SKY, disk_write_max, tc), disk_chunks[1]);

            let mut disk_rows = vec![];
            let disk_header = Row::new(vec!["Device", "Mount", "FS", "Total", "Available", "Used%"])
                .style(Style::default().fg(resolve(COLOR_PEACH, tc)).add_modifier(Modifier::BOLD))
                .bottom_margin(1);
            for (i, disk) in app.disks.iter().enumerate() {
                let total_gb = disk.total_space() as f64 / 1024.0 / 1024.0 / 1024.0;
                let avail_gb = disk.available_space() as f64 / 1024.0 / 1024.0 / 1024.0;
                let used_pct = if disk.total_space() > 0 {
                    (disk.total_space() - disk.available_space()) as f64 / disk.total_space() as f64 * 100.0
                } else { 0.0 };
                let name = disk.name().to_string_lossy().to_string();
                let mount = disk.mount_point().to_string_lossy().to_string();
                let fs = String::from_utf8_lossy(disk.file_system().as_encoded_bytes()).to_string();
                let row_color = if i % 2 == 0 { resolve(COLOR_TEXT, tc) } else { resolve(COLOR_SUBTEXT0, tc) };
                disk_rows.push(Row::new(vec![
                    name,
                    mount,
                    fs,
                    format!("{:.1} GB", total_gb),
                    format!("{:.1} GB", avail_gb),
                    format!("{:.1}%", used_pct),
                ]).style(Style::default().fg(row_color)));
            }
            if disk_rows.is_empty() {
                disk_rows.push(Row::new(vec!["No disks found", "", "", "", "", ""]).style(Style::default().fg(resolve(COLOR_SUBTEXT0, tc))));
            }
            let disk_table = Table::new(disk_rows, [
                Constraint::Percentage(20),
                Constraint::Percentage(20),
                Constraint::Percentage(10),
                Constraint::Percentage(15),
                Constraint::Percentage(15),
                Constraint::Percentage(20),
            ])
                .header(disk_header)
                .block(create_block(" Disk Partitions ", tc))
                .row_highlight_style(Style::default().bg(resolve(COLOR_SURFACE1, tc)).fg(resolve(COLOR_LAVENDER, tc)).add_modifier(Modifier::BOLD));
            f.render_widget(disk_table, disk_chunks[2]);
        }
        _ => {}
    }

    let help_line = match app.active_tab {
        0 => {
            if app.search_query.is_empty() {
                "[Tab] Next | [1-5] Jump | [↑↓] Nav | [/] Search | [s] Sort | [x] Kill | [e] Export | [q] Quit".to_string()
            } else {
                "[Tab] Next | [1-5] Jump | [↑↓] Nav | [/] Search | [Ctrl+C] Clear | [s] Sort | [x] Kill | [e] Export | [q] Quit".to_string()
            }
        }
        _ => "[Tab] Next View | [1-5] Jump | [e] Export | [q] Quit".to_string(),
    };

    let footer_chunks = if app.export_message.is_some() {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(0), Constraint::Length(50)])
            .split(main_chunks[1])
    } else {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(0), Constraint::Length(0)])
            .split(main_chunks[1])
    };

    f.render_widget(Paragraph::new(format!(" {} ", help_line)).block(create_block(" Help & Controls ", tc)).style(Style::default().fg(resolve(COLOR_SUBTEXT0, tc))), footer_chunks[0]);

    if let Some(ref msg) = app.export_message {
        f.render_widget(
            Paragraph::new(format!(" {} ", msg))
                .block(create_block_colored(" Exported ", COLOR_GREEN, tc))
                .style(Style::default().fg(resolve(COLOR_GREEN, tc))),
            footer_chunks[1],
        );
    }

    if app.show_sort_menu {
        let area = centered_rect(30, 25, f.area());
        f.render_widget(Clear, area);

        let menu_items = vec![
            " 1. CPU Usage ",
            " 2. Memory Usage ",
            " 3. Disk Read ",
            " 4. Disk Write ",
            "",
            " [Esc/s] Close ",
        ].join("\n");

        let popup_block = create_block_colored(" Sort Processes By ", COLOR_PEACH, tc);
        f.render_widget(Paragraph::new(menu_items).block(popup_block).style(Style::default().fg(resolve(COLOR_TEXT, tc))), area);
    }

    if app.search_mode {
        let area = centered_rect(40, 25, f.area());
        f.render_widget(Clear, area);

        let cursor = if (std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .subsec_millis()) < 500 { "█" } else { " " };

        let search_content = vec![
            String::new(),
            format!("  > {}{}", app.search_query, cursor),
            String::new(),
            format!("  {} result(s) found", app.filtered_processes.len()),
            String::new(),
            "  [Enter/Esc] Close | [Backspace] Delete".to_string(),
            "  [Ctrl+C] Clear & Close".to_string(),
        ].join("\n");

        let popup_block = create_block_colored(" Search Processes ", COLOR_TEAL, tc);
        f.render_widget(Paragraph::new(search_content).block(popup_block).style(Style::default().fg(resolve(COLOR_TEXT, tc))), area);
    }

    if app.show_kill_confirm {
        let area = centered_rect(40, 25, f.area());
        f.render_widget(Clear, area);

        let process_name = app.pending_kill_name.as_deref()
            .zip(app.pending_kill_pid)
            .map(|(name, pid)| format!(" \"{}\" (PID: {})", name, pid))
            .unwrap_or_default();

        let confirm_text = vec![
            String::new(),
            format!(" Kill process{} ?", process_name),
            String::new(),
            " [y] Yes, kill it".to_string(),
            " [n / Esc] Cancel".to_string(),
        ].join("\n");

        let popup_block = create_block_colored(" Confirm Kill ", COLOR_RED, tc);
        f.render_widget(Paragraph::new(confirm_text).block(popup_block).style(Style::default().fg(resolve(COLOR_TEXT, tc))), area);
    }
}
