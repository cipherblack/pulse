mod cli;
mod display;
mod process;
mod backup;
mod email;
mod stats;

use cli::{Cli, Commands};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Gauge, Paragraph, BarChart, Table, Row, Cell},
    Terminal,
};
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io;
use sysinfo::{System, SystemExt, DiskExt, ComponentExt, ProcessExt, CpuExt};
use tokio::time::{sleep, Duration};
use stats::Stats;
use clap::Parser;
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<(), io::Error> {
    let cli = Cli::parse();
    let mut sys = System::new_all();
    let mut stats = Stats::new(60);

    match cli.command {
        Commands::Monitor { interval, backup_interval, email } => {
            enable_raw_mode()?;
            let mut stdout = io::stdout();
            execute!(stdout, EnterAlternateScreen)?;
            let backend = CrosstermBackend::new(stdout);
            let mut terminal = Terminal::new(backend)?;

            let mut last_backup = Instant::now();
            let mut last_email = Instant::now();

            let res = run_monitor(
                &mut terminal,
                &mut sys,
                &mut stats,
                interval,
                backup_interval,
                email,
                &mut last_backup,
                &mut last_email,
            ).await;

            disable_raw_mode()?;
            execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
            terminal.show_cursor()?;
            res?;
        }
        Commands::Report => {
            sys.refresh_all();
            display::show_report(&sys);
        }
    }
    Ok(())
}

async fn run_monitor(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    sys: &mut System,
    stats: &mut Stats,
    interval: u64,
    backup_interval: u64,
    email: Option<String>,
    last_backup: &mut Instant,
    last_email: &mut Instant,
) -> io::Result<()> {
    let refresh_interval = Duration::from_millis(100);

    loop {
        sys.refresh_all();
        let cpu_usage = sys.global_cpu_info().cpu_usage();
        stats.update(cpu_usage);

        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([
                    Constraint::Length(3),      // Time
                    Constraint::Length(5),      // CPU & RAM side by side
                    Constraint::Percentage(30), // Disks
                    Constraint::Percentage(25), // Hardware & Status combined
                    Constraint::Percentage(40), // Processes (last section)
                ])
                .split(f.size());

            // Time
            let time = chrono::Local::now().format("%Y-%m-%d %H:%M:%S %z").to_string();
            let time_block = Paragraph::new(format!("=== SysPulse ===\nTime: {}", time))
                .style(Style::default().fg(Color::Cyan))
                .block(Block::default().borders(Borders::ALL));
            f.render_widget(time_block, chunks[0]);

            // CPU and RAM side by side
            let cpu_ram_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(50), // CPU
                    Constraint::Percentage(50), // RAM
                ])
                .split(chunks[1]);

            // CPU Usage
            let cpu_color = if cpu_usage > 90.0 { Color::Red } else if cpu_usage > 80.0 { Color::LightRed } else { Color::Green };
            let cpu_gauge = Gauge::default()
                .block(Block::default().title("CPU Usage").borders(Borders::ALL))
                .gauge_style(Style::default().fg(cpu_color))
                .percent(cpu_usage as u16)
                .label(format!("{:.2}%", cpu_usage));
            f.render_widget(cpu_gauge, cpu_ram_chunks[0]);

            // RAM Usage
            let total_mem = sys.total_memory() / 1024 / 1024;
            let used_mem = sys.used_memory() / 1024 / 1024;
            let ram_gauge = Gauge::default()
                .block(Block::default().title("RAM").borders(Borders::ALL))
                .gauge_style(Style::default().fg(Color::Yellow))
                .percent(((used_mem as f64 / total_mem as f64) * 100.0) as u16)
                .label(format!("{} MB / {} MB", used_mem, total_mem));
            f.render_widget(ram_gauge, cpu_ram_chunks[1]);

            // Disks
            let mut disks: Vec<_> = sys.disks().iter().map(|disk| {
                let total = disk.total_space() / 1_073_741_824;
                let free = disk.available_space() / 1_073_741_824;
                let used = total - free;
                let is_mounted = disk.is_removable() || disk.available_space() > 0;
                (disk.mount_point().to_string_lossy().to_string(), used as u64, total as u64, free as u64, is_mounted)
            }).collect();
            disks.sort_by(|a, b| a.0.cmp(&b.0));
            let disk_texts: Vec<_> = disks.iter().enumerate().map(|(_, (name, used, total, free, is_mounted))| {
                format!("{}: {}/{} GB (Free: {} GB, {})", name, used, total, free, if *is_mounted { "In Use" } else { "Not Mounted" })
            }).collect();
            let disk_bars = BarChart::default()
                .block(Block::default().title("Disks").borders(Borders::ALL))
                .data(&disks.iter().map(|(name, used, _, _, _)| (name.as_str(), *used)).collect::<Vec<_>>())
                .bar_width(12)
                .max(200)
                .bar_style(Style::default().fg(Color::Blue));
            f.render_widget(disk_bars, chunks[2]);
            let disk_details = Paragraph::new(disk_texts.join("\n"))
                .style(Style::default().fg(Color::White))
                .block(Block::default().title("Disk Details").borders(Borders::ALL));
            f.render_widget(disk_details, chunks[2]);

            // Combined Hardware & Status
            let mut hw_status_text = Vec::new();
            // Hardware info
            if let Some(cpu) = sys.cpus().first() {
                hw_status_text.push(format!("CPU: {}", cpu.brand()));
            }
            hw_status_text.push(format!("Cores: {}", sys.physical_core_count().unwrap_or(0)));
            for component in sys.components() {
                hw_status_text.push(format!("{}: {:.1}°C", component.label(), component.temperature()));
            }
            // Status info
            if let Some(avg) = stats.average_cpu() {
                hw_status_text.push(format!("Avg CPU (5 min): {:.2}%", avg));
            }
            if let Some(trend) = stats.cpu_trend() {
                if trend > 0.0 {
                    hw_status_text.push(format!("CPU Trend: Increasing (+{:.2}%)", trend));
                }
            }
            if cpu_usage > 90.0 {
                hw_status_text.push("CRITICAL: CPU usage exceeds 90!".to_string());
            }
            let hw_status = Paragraph::new(hw_status_text.join("\n"))
                .style(Style::default().fg(Color::LightGreen))
                .block(Block::default().title("Hardware & Status").borders(Borders::ALL));
            f.render_widget(hw_status, chunks[3]);

            // Processes (sorted by CPU usage)
            let mut processes: Vec<_> = sys.processes()
                .iter()
                .map(|(pid, proc)| {
                    (proc.cpu_usage(), Row::new(vec![
                        Cell::from(proc.name().to_string()),
                        Cell::from(pid.to_string()),
                        Cell::from(proc.exe().to_string_lossy().to_string()),
                        Cell::from(format!("{:.2}%", proc.cpu_usage())),
                        Cell::from(format!("{} MB", proc.memory() / 1024 / 1024)),
                    ]))
                })
                .collect();
            processes.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
            let process_table = Table::new(
                processes.into_iter().map(|(_, row)| row),
                &[
                    Constraint::Percentage(30),
                    Constraint::Length(10),
                    Constraint::Percentage(40),
                    Constraint::Length(10),
                    Constraint::Length(10),
                ]
            )
                .header(Row::new(vec!["Name", "PID", "Path", "CPU", "RAM"])
                    .style(Style::default().fg(Color::Yellow)))
                .block(Block::default().title("Processes").borders(Borders::ALL))
                .column_spacing(1)
                .style(Style::default().fg(Color::White));
            f.render_widget(process_table, chunks[4]);
        })?;

        process::manage(sys, cpu_usage);
        if last_backup.elapsed().as_secs() >= backup_interval {
            backup::save(sys);
            *last_backup = Instant::now();
        }
        if let Some(email_addr) = &email {
            if cpu_usage > 80.0 && last_email.elapsed().as_secs() > 300 {
                email::send(email_addr, cpu_usage).await;
                *last_email = Instant::now();
            }
        }

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') {
                    return Ok(());
                }
            }
        }

        sleep(refresh_interval).await;
    }
}