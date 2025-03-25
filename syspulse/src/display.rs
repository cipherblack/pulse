use sysinfo::{System, SystemExt, CpuExt, DiskExt};
use colored::Colorize;
use crate::stats::Stats;

#[allow(dead_code)]
pub fn clear_screen() {
  #[cfg(target_os = "windows")]
  std::process::Command::new("cmd").arg("/c").arg("cls").status().unwrap();
  
  #[cfg(not(target_os = "windows"))]
  std::process::Command::new("clear").status().unwrap();
}

#[allow(dead_code)]
pub fn show_stats(sys: &System, stats: &Stats) {
  let cpu_usage = sys.global_cpu_info().cpu_usage();
  
  println!("{}", "=== SysPulse ===".bold().cyan());
  println!("Time: {}", chrono::Local::now().to_string().white());
  
  println!("CPU Usage: {}", if cpu_usage > 90.0 {
    format!("{:.2}%", cpu_usage).red().bold()
  } else if cpu_usage > 80.0 {
    format!("{:.2}%", cpu_usage).red()
  } else {
    format!("{:.2}%", cpu_usage).green()
  });
  
  println!("RAM: {} MB / {} MB", format!("{}", sys.used_memory() / 1024 / 1024).yellow(), format!("{}", sys.total_memory() / 1024 / 1024).yellow());
  
  for disk in sys.disks() {
    println!("Disk {}: {} GB free / {} GB total", disk.mount_point().to_string_lossy().blue(), format!("{}", disk.available_space() / 1024 / 1024 / 1024).green(), format!("{}", disk.total_space() / 1024 / 1024 / 1024).green());
  }
  
  if let Some(avg) = stats.average_cpu() {
    println!("Avg CPU (5 min): {:.2}%", format!("{:.2}%", avg).purple());
  }
  
  if let Some(trend) = stats.cpu_trend() {
    if trend > 0.0 {
      println!("CPU Trend: Increasing (+{:.2}%)", format!("{:.2}%", trend).purple());
    }
  }
  
  if cpu_usage > 90.0 {
    println!("{}", "CRITICAL: CPU usage exceeds 90%!".red().bold());
  }
}

pub fn show_report(sys: &System) {
  let report = serde_json::json!({
    "timestamp": chrono::Local::now().to_string(),
    "cpu_usage": sys.global_cpu_info().cpu_usage(),
    "memory": {
      "used_mb": sys.used_memory() / 1024 / 1024,
      "total_mb": sys.total_memory() / 1024 / 1024
    },
    "disks": sys.disks().iter().map(|d| {
      serde_json::json!({
        "mount": d.mount_point().to_string_lossy(),
        "free_gb": d.available_space() / 1024 / 1024 / 1024,
        "total_gb": d.total_space() / 1024 / 1024 / 1024
      })
    }).collect::<Vec<_>>()
  });
  
  println!("{}", serde_json::to_string_pretty(&report).unwrap());
}