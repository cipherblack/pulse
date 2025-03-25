use sysinfo::{System, ProcessExt, Pid, SystemExt};
use colored::Colorize;
use std::io::{Write, stdin};

pub fn manage(sys: &mut System, cpu_usage: f32) {
  if cpu_usage > 90.0 {
    println!("{}", "High CPU detected! Checking processes...".yellow().bold());
    sys.refresh_processes();

    let mut heavy_processes: Vec<(Pid, &sysinfo::Process)> = sys.processes()
      .iter()
      .filter(|(_, p)| p.cpu_usage() > 10.0)
      .map(|(pid, p)| (*pid, p))
      .collect();

    heavy_processes.sort_by(|a, b| b.1.cpu_usage().partial_cmp(&a.1.cpu_usage()).unwrap());

    for (pid, proc) in heavy_processes.iter().take(3) {
      println!(
        "{} Process: {} (PID: {}) - CPU: {:.2}% - Running for: {}s",
        "Heavy".red(),
        proc.name().white(),
        pid,
        format!("{:.2}%", proc.cpu_usage()).red(),
        proc.run_time()
      );

      if prompt_user(format!("Kill process {} (PID: {})? [y/n]: ", proc.name(), pid)) {
        if sys.process(*pid).unwrap().kill() {
          println!("{}", "Process killed!".green());
        } else {
          println!("{}", "Failed to kill process!".red());
        }
      }
    }

    let old_processes: Vec<(Pid, &sysinfo::Process)> = sys.processes()
      .iter()
      .filter(|(_, p)| p.run_time() > 3600 && p.cpu_usage() < 1.0)
      .map(|(pid, p)| (*pid, p))
      .collect();

    for (pid, proc) in old_processes.iter().take(3) {
      println!(
        "{} Process: {} (PID: {}) - CPU: {:.2}% - Running for: {}s",
        "Idle".yellow(),
        proc.name().white(),
        pid,
        format!("{:.2}%", proc.cpu_usage()).yellow(),
        proc.run_time()
      );

      if prompt_user(format!("Kill idle process {} (PID: {})? [y/n]: ", proc.name(), pid)) {
        if sys.process(*pid).unwrap().kill() {
          println!("{}", "Idle process killed!".green());
        }
      }
    }
  }
}

fn prompt_user(prompt: String) -> bool {
  print!("{}", prompt);
  std::io::stdout().flush().unwrap();
  let mut input = String::new();
  stdin().read_line(&mut input).unwrap();
  input.trim().to_lowercase() == "y"
}