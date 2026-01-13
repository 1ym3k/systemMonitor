
## Sysmon
Sysmon (systemMonitor) — a simple, educational system monitor written in Rust for Linux / WSL.

This project was created for learning purposes. It allows user to monitor system statistics such as CPU usage, memory, and process information. It is an educational project and not intended for professional use.

---

## Features

* **Real-time CPU Monitoring:** Track load and core performance.
* **RAM Usage:** Visual representation of memory consumption.
* **Network Speed:** Monitor incoming and outgoing data.
* **Process Management:** View top 10 processes by RAM usage.
* **Rust-powered:** Built for safety and performance.

---

## Requirements

* **Rust** (stable)
* **Cargo** to build and run
* **Linux** or **WSL** environment

---

## Installation

1. **Clone the repository:**
   ```bash
   git clone [https://github.com/1ym3k/systemMonitor.git](https://github.com/1ym3k/systemMonitor.git)
   cd systemMonitor
   ```
2. Build the project:
   ```bash
   cargo build --release
   ```
3. Run:
   ```bash
   cargo run --release
   ```
## How it works
The project uses Rust tools to read system metrics and display them in real-time. The interface is inspired by modern RPG HUDs seen in modern titles like Final Fantasy XV.

## Credits
Author: 1ym3k
Font: TeX Gyre Adventor by GUST e-foundry (B. Jackowski and J. M. Nowacki).
Art: Logo by Yoshitaka Amano for Square Enix.
Inspiration: Visual style inspired by Final Fantasy XV Windows Edition.
