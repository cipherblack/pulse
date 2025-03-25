use sysinfo::{System, SystemExt, DiskExt, CpuExt};
use std::fs::OpenOptions;
use std::io::Write;
use colored::Colorize;

pub fn save(sys: &System) {
  let report = serde_json::json!({
    "timestamp": chrono::Local::now().to_string(),
    "cpu_usage": sys.global_cpu_info().cpu_usage(),
    "memory_used_mb": sys.used_memory() / 1024 / 1024,
    "memory_total_mb": sys.total_memory() / 1024 / 1024,
    "disks_free_gb": sys.disks().iter().map(|d| d.available_space() / 1024 / 1024 / 1024).collect::<Vec<_>>()
  });

  let mut file = OpenOptions::new()
    .append(true)
    .create(true)
    .open("syspulse_backup.json")
    .unwrap();

  writeln!(file, "{}", serde_json::to_string(&report).unwrap()).unwrap();

  println!("{}", "Backup saved!".green());
}