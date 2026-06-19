use sysinfo::{System, Networks, Disks, Components, Users, Pid};
use std::collections::VecDeque;
use ratatui::widgets::TableState;
use std::thread;
use std::time::{Duration, Instant};
use std::fs;
use anyhow::Result;

use crate::cli::Config;

pub const TAB_COUNT: usize = 5;

pub enum SortBy {
    Cpu,
    Memory,
    DiskRead,
    DiskWrite,
}

pub struct App {
    pub sys: System,
    pub networks: Networks,
    pub disks: Disks,
    pub components: Components,
    pub users: Users,
    pub cpu_history: VecDeque<f32>,
    pub per_cpu_history: Vec<VecDeque<f32>>,
    pub mem_history: VecDeque<f32>,
    pub swap_history: VecDeque<f32>,
    pub network_in_history: VecDeque<f64>,
    pub network_out_history: VecDeque<f64>,
    pub disk_read_history: VecDeque<f64>,
    pub disk_write_history: VecDeque<f64>,
    pub processes: Vec<ProcessInfo>,
    pub filtered_processes: Vec<usize>,
    pub network_in: u64,
    pub network_out: u64,
    pub load_avg: (f64, f64, f64),
    pub table_state: TableState,
    pub active_tab: usize,
    pub max_cpu_freq_static: String,
    pub cpu_governor: String,
    pub uptime: u64,
    pub current_cpu_freq: u64,
    pub l1_cache: String,
    pub l2_cache: String,
    pub l3_cache: String,
    pub sort_by: SortBy,
    pub show_sort_menu: bool,
    pub show_kill_confirm: bool,
    pub pending_kill_pid: Option<u32>,
    pub pending_kill_name: Option<String>,
    pub search_mode: bool,
    pub search_query: String,
    pub alert_cpu: bool,
    pub alert_mem: bool,
    pub no_truecolor: bool,
    pub export_message: Option<String>,
    export_message_at: Option<Instant>,
    last_network_update: Instant,
    last_disk_update: Instant,
}

#[derive(Clone)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub user: String,
    pub cpu: f32,
    pub mem: u64,
    pub disk_read: u64,
    pub disk_write: u64,
}

fn get_linux_max_cpu_freq() -> String {
    if let Some(mhz) = fs::read_to_string("/sys/devices/system/cpu/cpu0/cpufreq/cpuinfo_max_freq")
        .ok()
        .and_then(|s| s.trim().parse::<f64>().ok())
        .map(|khz| khz / 1000.0)
    {
        if mhz >= 1000.0 {
            return format!("{:.2} GHz", mhz / 1000.0);
        } else {
            return format!("{:.0} MHz", mhz);
        }
    }
    "Unknown".to_string()
}

fn get_linux_cpu_governor() -> String {
    if let Ok(gov) = fs::read_to_string("/sys/devices/system/cpu/cpu0/cpufreq/scaling_governor") {
        return gov.trim().to_string();
    }
    "Unknown".to_string()
}

fn get_linux_cpu_cache() -> (String, String, String) {
    let mut l1_total_k = 0;
    let mut l2 = String::from("Unknown");
    let mut l3 = String::from("Unknown");

    for i in 0..5 {
        let base_path = format!("/sys/devices/system/cpu/cpu0/cache/index{}", i);
        let level = fs::read_to_string(format!("{}/level", base_path));
        let size = fs::read_to_string(format!("{}/size", base_path));
        if let (Ok(level), Ok(size)) = (level, size) {
            let lvl = level.trim();
            let sz = size.trim();
            match lvl {
                "1" => {
                    if let Ok(k) = sz.replace("K", "").parse::<u32>() {
                        l1_total_k += k;
                    }
                },
                "2" => l2 = sz.to_string(),
                "3" => l3 = sz.to_string(),
                _ => {}
            }
        }
    }

    let l1_str = if l1_total_k > 0 { format!("{}K", l1_total_k) } else { "Unknown".to_string() };
    (l1_str, l2, l3)
}

impl App {
    pub fn new(config: &Config) -> Self {
        let mut sys = System::new_all();
        sys.refresh_cpu_all();
        thread::sleep(Duration::from_millis(100));
        sys.refresh_all();

        let networks = Networks::new_with_refreshed_list();
        let disks = Disks::new_with_refreshed_list();
        let components = Components::new_with_refreshed_list();
        let users = Users::new_with_refreshed_list();
        let mut table_state = TableState::default();
        table_state.select(Some(0));

        let num_cpus = sys.cpus().len();
        let mut per_cpu_history = Vec::with_capacity(num_cpus);
        for _ in 0..num_cpus {
            per_cpu_history.push(VecDeque::with_capacity(1000));
        }

        let (l1, l2, l3) = get_linux_cpu_cache();

        let mut app = App {
            sys,
            networks,
            disks,
            components,
            users,
            cpu_history: VecDeque::with_capacity(1000),
            per_cpu_history,
            mem_history: VecDeque::with_capacity(1000),
            swap_history: VecDeque::with_capacity(1000),
            network_in_history: VecDeque::with_capacity(1000),
            network_out_history: VecDeque::with_capacity(1000),
            disk_read_history: VecDeque::with_capacity(1000),
            disk_write_history: VecDeque::with_capacity(1000),
            processes: Vec::new(),
            filtered_processes: Vec::new(),
            network_in: 0,
            network_out: 0,
            load_avg: (0.0, 0.0, 0.0),
            table_state,
            active_tab: 0,
            max_cpu_freq_static: get_linux_max_cpu_freq(),
            cpu_governor: get_linux_cpu_governor(),
            uptime: 0,
            current_cpu_freq: 0,
            l1_cache: l1,
            l2_cache: l2,
            l3_cache: l3,
            sort_by: SortBy::Cpu,
            show_sort_menu: false,
            show_kill_confirm: false,
            pending_kill_pid: None,
            pending_kill_name: None,
            search_mode: false,
            search_query: config.initial_filter.clone(),
            alert_cpu: false,
            alert_mem: false,
            no_truecolor: config.no_truecolor,
            export_message: None,
            export_message_at: None,
            last_network_update: Instant::now(),
            last_disk_update: Instant::now(),
        };

        app.update();
        app
    }

    pub fn update_filter(&mut self) {
        let query = self.search_query.to_lowercase();
        self.filtered_processes = if query.is_empty() {
            (0..self.processes.len()).collect()
        } else {
            self.processes.iter().enumerate().filter_map(|(idx, p)| {
                if p.name.to_lowercase().contains(&query) || p.pid.to_string().contains(&query) {
                    Some(idx)
                } else {
                    None
                }
            }).collect()
        };

        let selected = self.table_state.selected().unwrap_or(0);
        if self.filtered_processes.is_empty() {
            self.table_state.select(None);
        } else if selected >= self.filtered_processes.len() {
            self.table_state.select(Some(0));
        }
    }

    pub fn update(&mut self) {
        self.sys.refresh_all();

        let net_elapsed = self.last_network_update.elapsed().as_secs_f64().max(0.001);
        self.networks.refresh(true);
        self.last_network_update = Instant::now();

        let disk_elapsed = self.last_disk_update.elapsed().as_secs_f64().max(0.001);
        self.disks.refresh(true);
        self.last_disk_update = Instant::now();
        self.components.refresh(true);

        self.uptime = System::uptime();

        let la = System::load_average();
        self.load_avg = (la.one, la.five, la.fifteen);

        let cpus = self.sys.cpus();
        if !cpus.is_empty() {
            let avg_freq: u64 = cpus.iter().map(|c| c.frequency()).sum::<u64>() / cpus.len() as u64;
            self.current_cpu_freq = avg_freq;

            let avg_cpu: f32 = cpus.iter().map(|cpu| cpu.cpu_usage()).sum::<f32>() / cpus.len() as f32;
            self.cpu_history.push_back(avg_cpu);
            self.alert_cpu = avg_cpu > 90.0;

            for (i, cpu) in cpus.iter().enumerate() {
                if i < self.per_cpu_history.len() {
                    self.per_cpu_history[i].push_back(cpu.cpu_usage());
                    if self.per_cpu_history[i].len() > 1000 {
                        self.per_cpu_history[i].pop_front();
                    }
                }
            }
        }
        if self.cpu_history.len() > 1000 { self.cpu_history.pop_front(); }

        if self.sys.total_memory() > 0 {
            let used_actual = self.sys.total_memory().saturating_sub(self.sys.available_memory());
            let used_percent = used_actual as f32 / self.sys.total_memory() as f32 * 100.0;
            self.mem_history.push_back(used_percent);
            self.alert_mem = used_percent > 85.0;
        }
        if self.mem_history.len() > 1000 { self.mem_history.pop_front(); }

        if self.sys.total_swap() > 0 {
            let swap_percent = self.sys.used_swap() as f32 / self.sys.total_swap() as f32 * 100.0;
            self.swap_history.push_back(swap_percent);
        } else {
            self.swap_history.push_back(0.0);
        }
        if self.swap_history.len() > 1000 { self.swap_history.pop_front(); }

        let mut total_disk_read = 0u64;
        let mut total_disk_write = 0u64;
        for disk in &self.disks {
            total_disk_read += disk.usage().read_bytes;
            total_disk_write += disk.usage().written_bytes;
        }
        let disk_read_kbps = total_disk_read as f64 / disk_elapsed / 1024.0;
        let disk_write_kbps = total_disk_write as f64 / disk_elapsed / 1024.0;
        self.disk_read_history.push_back(disk_read_kbps);
        if self.disk_read_history.len() > 1000 { self.disk_read_history.pop_front(); }
        self.disk_write_history.push_back(disk_write_kbps);
        if self.disk_write_history.len() > 1000 { self.disk_write_history.pop_front(); }

        let user_map: std::collections::HashMap<&sysinfo::Uid, String> = self.users
            .iter()
            .map(|user| (user.id(), user.name().to_string()))
            .collect();

        self.processes = self.sys.processes().iter().map(|(pid, process)| {
            let du = process.disk_usage();
            let user = process.user_id()
                .and_then(|uid| user_map.get(uid))
                .cloned()
                .unwrap_or_default();
            ProcessInfo {
                pid: pid.as_u32(),
                name: process.name().to_string_lossy().to_string(),
                user,
                cpu: process.cpu_usage(),
                mem: process.memory(),
                disk_read: du.read_bytes,
                disk_write: du.written_bytes,
            }
        }).collect();

        match self.sort_by {
            SortBy::Cpu => self.processes.sort_unstable_by(|a, b| b.cpu.partial_cmp(&a.cpu).unwrap_or(std::cmp::Ordering::Equal)),
            SortBy::Memory => self.processes.sort_unstable_by_key(|b| std::cmp::Reverse(b.mem)),
            SortBy::DiskRead => self.processes.sort_unstable_by_key(|b| std::cmp::Reverse(b.disk_read)),
            SortBy::DiskWrite => self.processes.sort_unstable_by_key(|b| std::cmp::Reverse(b.disk_write)),
        }

        self.update_filter();

        let mut total_in = 0u64;
        let mut total_out = 0u64;
        for (_interface_name, data) in &self.networks {
            total_in += data.received();
            total_out += data.transmitted();
        }
        self.network_in = total_in;
        self.network_out = total_out;

        let in_kbps = total_in as f64 / net_elapsed / 1024.0;
        let out_kbps = total_out as f64 / net_elapsed / 1024.0;

        self.network_in_history.push_back(in_kbps);
        if self.network_in_history.len() > 1000 { self.network_in_history.pop_front(); }
        self.network_out_history.push_back(out_kbps);
        if self.network_out_history.len() > 1000 { self.network_out_history.pop_front(); }

        if self.export_message_at.is_some_and(|at| at.elapsed().as_secs() >= 3) {
            self.export_message = None;
            self.export_message_at = None;
        }
    }

    pub fn next_tab(&mut self) {
        self.active_tab = (self.active_tab + 1) % TAB_COUNT;
    }

    pub fn next_process(&mut self) {
        if self.filtered_processes.is_empty() { return; }
        let i = match self.table_state.selected() {
            Some(i) => if i >= self.filtered_processes.len() - 1 { i } else { i + 1 },
            None => 0,
        };
        self.table_state.select(Some(i));
    }

    pub fn previous_process(&mut self) {
        if self.filtered_processes.is_empty() { return; }
        let i = match self.table_state.selected() {
            Some(i) => if i == 0 { 0 } else { i - 1 },
            None => 0,
        };
        self.table_state.select(Some(i));
    }

    pub fn kill_selected(&mut self) {
        if let Some(p) = self.table_state.selected()
            .and_then(|i| self.filtered_processes.get(i))
            .and_then(|&idx| self.processes.get(idx))
        {
            self.pending_kill_pid = Some(p.pid);
            self.pending_kill_name = Some(p.name.clone());
            self.show_kill_confirm = true;
        }
    }

    pub fn confirm_kill(&mut self) {
        if let Some(pid) = self.pending_kill_pid {
            let sysinfo_pid = Pid::from_u32(pid);
            if let Some(process) = self.sys.process(sysinfo_pid) {
                let _ = process.kill();
            }
        }
        self.show_kill_confirm = false;
        self.pending_kill_pid = None;
        self.pending_kill_name = None;
    }

    pub fn cancel_kill(&mut self) {
        self.show_kill_confirm = false;
        self.pending_kill_pid = None;
        self.pending_kill_name = None;
    }

    pub fn search_push(&mut self, c: char) {
        self.search_query.push(c);
        self.update_filter();
    }

    pub fn search_pop(&mut self) {
        self.search_query.pop();
        self.update_filter();
    }

    pub fn search_clear(&mut self) {
        self.search_query.clear();
        self.search_mode = false;
        self.update_filter();
    }

    pub fn export_snapshot(&mut self) -> Result<()> {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let path = format!("{}/resource-monitor-{}.csv", home, timestamp);

        let mut csv = String::from("pid,name,cpu_percent,mem_mb,disk_read_bytes,disk_write_bytes\n");
        for p in &self.processes {
            csv.push_str(&format!(
                "{},{},{:.1},{:.1},{},{}\n",
                p.pid,
                p.name,
                p.cpu,
                p.mem as f64 / 1024.0 / 1024.0,
                p.disk_read,
                p.disk_write,
            ));
        }

        fs::write(&path, csv)?;
        self.export_message = Some(format!("Exported to {}", path));
        self.export_message_at = Some(Instant::now());
        Ok(())
    }
}
