use macroquad::prelude::*;
use std::fs::{read_to_string, read_dir};
use std::{thread, time::Duration};
use chrono::Local;

// Uniwersalna funkcja do wyciągania wartości z plików systemowych
fn get_info(path: &str, key: &str) -> String {
    if let Ok(content) = read_to_string(path) {
        for line in content.lines() {
            if line.to_lowercase().contains(&key.to_lowercase()) {
                let delimiter = if line.contains(':') { ':' } else { '=' };
                if let Some(value) = line.split(delimiter).nth(1) {
                    return value.trim().replace('"', "").to_string();
                }
            }
        }
    }
    "N/A".to_string()
}

// Funkcja odczytu temperatury
fn read_temp() -> String {
    for i in 0..5 {
        let path = format!("/sys/class/thermal/thermal_zone{}/temp", i);
        if let Ok(t) = read_to_string(path) {
            let temp_c = t.trim().parse::<f32>().unwrap_or(0.0) / 1000.0;
            if temp_c > 0.0 { return format!("{:.1}°C", temp_c); }
        }
    }
    "N/A".to_string()
}

struct ProcessInfo {
    name: String,
    pid: String,
    ram_mb: f32,
}

fn get_top_processes() -> Vec<ProcessInfo> {
    let mut procs = Vec::new();
    if let Ok(entries) = read_dir("/proc") {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.chars().all(|c| c.is_numeric()) {
                let status_path = format!("/proc/{}/status", name);
                let proc_name = get_info(&status_path, "Name");
                let ram_kb_str = get_info(&status_path, "VmRSS");
                let ram_kb = ram_kb_str.split_whitespace().next().unwrap_or("0").parse::<f32>().unwrap_or(0.0);
                
                if ram_kb > 0.0 {
                    procs.push(ProcessInfo {
                        name: proc_name,
                        pid: name,
                        ram_mb: ram_kb / 1024.0,
                    });
                }
            }
        }
    }
    procs.sort_by(|a, b| b.ram_mb.partial_cmp(&a.ram_mb).unwrap_or(std::cmp::Ordering::Equal));
    procs.truncate(10);
    procs
}

fn window_conf() -> Conf {
    Conf {
        window_title: "SYSMON XIII VERSUS".to_owned(),
        window_width: 1000,
        window_height: 950,
        window_resizable: false,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    // --- ŁADOWANIE ZASOBÓW ---
    let logo_texture = load_texture("assets/logo.png").await.expect("Failed to load logo.png");
    let bg_texture = load_texture("assets/background.png").await.expect("Failed to load background.png");
    let my_font = load_ttf_font("assets/my_font.otf").await.expect("Failed to load font");

    let mut cpu_history = vec![0.0; 200];
    let mut ram_history = vec![0.0; 200];
    let mut net_history = vec![0.0; 200];
    
    let mut last_cpu_total = 0u64;
    let mut last_cpu_idle = 0u64;
    let mut last_net_bytes = 0u64;
    
    let host = hostname::get().unwrap_or_default().to_string_lossy().to_string();
    let cpu_model = get_info("/proc/cpuinfo", "model name");
    let os_name = get_info("/etc/os-release", "PRETTY_NAME");

    loop {
        // --- LOGIKA SYSTEMOWA ---
        let mut cpu_val = 0.0;
        if let Ok(stat) = read_to_string("/proc/stat") {
            let line = stat.lines().next().unwrap_or("");
            let parts: Vec<u64> = line.split_whitespace().skip(1).filter_map(|s| s.parse().ok()).collect();
            if parts.len() >= 4 {
                let idle = parts[3];
                let total: u64 = parts.iter().sum();
                let diff_total = total - last_cpu_total;
                let diff_idle = idle - last_cpu_idle;
                if diff_total > 0 {
                    cpu_val = 100.0 * (1.0 - (diff_idle as f32 / diff_total as f32));
                }
                last_cpu_total = total;
                last_cpu_idle = idle;
            }
        }
        cpu_history.remove(0);
        cpu_history.push(cpu_val);

        let mut ram_pct = 0.0;
        let mut ram_used_gb = 0.0;
        let mut total_ram_gb = 0.0;
        if let Ok(mem) = read_to_string("/proc/meminfo") {
            let mut t = 1.0; let mut a = 0.0;
            for l in mem.lines() {
                if l.starts_with("MemTotal:") { t = l.split_whitespace().nth(1).unwrap().parse().unwrap(); }
                if l.starts_with("MemAvailable:") { a = l.split_whitespace().nth(1).unwrap().parse().unwrap(); }
            }
            total_ram_gb = t / 1024.0 / 1024.0;
            ram_used_gb = (t - a) / 1024.0 / 1024.0;
            ram_pct = ((t - a) / t) * 100.0;
        }
        ram_history.remove(0);
        ram_history.push(ram_pct);

        let mut net_val = 0.0;
        if let Ok(net) = read_to_string("/proc/net/dev") {
            let mut cb = 0u64;
            for line in net.lines() {
                if line.contains("eth0") || line.contains("enp") || line.contains("wlan") {
                    cb = line.split_whitespace().nth(1).unwrap_or("0").parse().unwrap_or(0);
                }
            }
            if last_net_bytes > 0 && cb >= last_net_bytes {
                net_val = (cb - last_net_bytes) as f32 / 1024.0;
            }
            last_net_bytes = cb;
        }
        net_history.remove(0);
        net_history.push(net_val);

        let top_procs = get_top_processes();

        // --- RYSOWANIE ---
        
        // 1. Rysowanie tła PNG
        draw_texture_ex(
            &bg_texture,
            0.0,
            0.0,
            Color::new(0.6, 0.6, 0.6, 1.0), // Lekkie przyciemnienie tekstury (0.6 zamiast 1.0)
            DrawTextureParams {
                dest_size: Some(vec2(screen_width(), screen_height())),
                ..Default::default()
            },
        );

        let draw_ftext = |text: &str, x: f32, y: f32, size: f32, color: Color| {
            draw_text_ex(
                text,
                x,
                y,
                TextParams {
                    font: Some(&my_font),
                    font_size: size as u16,
                    color,
                    ..Default::default()
                },
            );
        };

        // 2. Logo
        draw_texture_ex(
            &logo_texture,
            550.0, 600.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(384.0, 290.0)),
                ..Default::default()
            },
        );
        
        // 3. Nagłówek
        let time_str = Local::now().format("%H:%M:%S").to_string();
        draw_ftext(&format!("TIME: {}", time_str), 820.0, 40.0, 25.0, WHITE);
        
        draw_ftext(&format!("SYSTEM ID: {}", host), 20.0, 45.0, 35.0, WHITE);
        draw_ftext(&format!("OS: {}", os_name), 20.0, 75.0, 20.0, WHITE);
        draw_ftext(&format!("CPU: {}", cpu_model), 20.0, 100.0, 18.0, WHITE);
        draw_ftext(&format!("TOTAL RAM: {:.2} GB", total_ram_gb), 20.0, 125.0, 18.0, WHITE);

        // 4. Wykresy (z lekkim półprzezroczystym tłem dla czytelności)
        draw_chart(&format!("CPU LOAD: {:.1}% | TEMP: {}", cpu_val, read_temp()), 170.0, &cpu_history, RED, 100.0, &my_font);
        draw_chart(&format!("RAM USAGE: {:.1}% ({:.2} GB used)", ram_pct, ram_used_gb), 320.0, &ram_history, SKYBLUE, 100.0, &my_font);
        draw_chart(&format!("NET SPEED: {:.1} KB/s", net_val * 10.0), 470.0, &net_history, PURPLE, 500.0, &my_font);

        // 5. Lista Procesów
        draw_ftext("TOP 10 PROCESSES (by RAM usage)", 20.0, 620.0, 25.0, WHITE);
        draw_ftext("NAME", 40.0, 650.0, 20.0, LIGHTGRAY);
        draw_ftext("PID", 250.0, 650.0, 20.0, LIGHTGRAY);
        draw_ftext("RAM USAGE", 350.0, 650.0, 20.0, LIGHTGRAY);
        draw_line(20.0, 655.0, 500.0, 655.0, 1.0, WHITE);

        for (i, p) in top_procs.iter().enumerate() {
            let y = 680.0 + (i as f32 * 22.0);
            draw_ftext(&p.name, 40.0, y, 18.0, WHITE);
            draw_ftext(&p.pid, 250.0, y, 18.0, LIGHTGRAY);
            draw_ftext(&format!("{:.2} MB", p.ram_mb), 350.0, y, 18.0, SKYBLUE);
        }

        // 6. Uptime & Stats
        let uptime = read_to_string("/proc/uptime").unwrap_or_default().split_whitespace().next().unwrap_or("0").parse::<f32>().unwrap_or(0.0);
        let procs_info = read_to_string("/proc/loadavg").unwrap_or_default().split_whitespace().nth(3).unwrap_or("N/A").to_string();
        draw_ftext(&format!("UPTIME: {:.0}s", uptime), 20.0, 930.0, 20.0, WHITE);
        draw_ftext(&format!("SYSTEM THREADS: {}", procs_info), 250.0, 930.0, 20.0, LIGHTGRAY);
        draw_ftext(&format!("Created by 1ym3k. Logo by Yoshitaka Amano for Square Enix. Font: TeX Gyre Adventor by GUST e-foundry."), 520.0, 940.0, 9.0, LIGHTGRAY);


        next_frame().await;
        thread::sleep(Duration::from_millis(100));
    }
}

fn draw_chart(label: &str, y: f32, data: &[f32], color: Color, max: f32, font: &Font) {
    let w = 960.0;
    let h = 80.0;
    
    draw_text_ex(
        label,
        20.0,
        y,
        TextParams {
            font: Some(font),
            font_size: 22,
            color: WHITE,
            ..Default::default()
        },
    );

    // Tło wykresu z lekką przezroczystością
    draw_rectangle(20.0, y + 10.0, w, h, Color::from_rgba(20, 20, 25, 180));
    
    for i in 0..data.len() - 1 {
        let x1 = 20.0 + (i as f32 * (w / 200.0));
        let x2 = 20.0 + ((i + 1) as f32 * (w / 200.0));
        let y1 = (y + 10.0 + h) - (data[i] / max * h).min(h);
        let y2 = (y + 10.0 + h) - (data[i+1] / max * h).min(h);
        draw_line(x1, y1, x2, y2, 2.0, color);
    }
}