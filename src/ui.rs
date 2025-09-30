use std::io::{self, Write};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};
use std::thread;
use std::time::Duration;

use crate::green;

pub struct CliUi {
    width: usize,
    current_msg: Arc<Mutex<String>>,
    spinner_active: Arc<AtomicBool>,
    progress_active: Arc<AtomicBool>,
    progress_msg: Arc<Mutex<String>>,
}

impl CliUi {
    pub fn new() -> Self {
        Self {
            width: 30,
            current_msg: Arc::new(Mutex::new(String::new())),
            spinner_active: Arc::new(AtomicBool::new(false)),
            progress_active: Arc::new(AtomicBool::new(false)),
            progress_msg: Arc::new(Mutex::new(String::new())),
        }
    }

    fn format_bytes(bytes: u64) -> String {
        const KB: f64 = 1024.0;
        const MB: f64 = KB * 1024.0;
        const GB: f64 = MB * 1024.0;

        let b = bytes as f64;
        if b >= GB {
            format!("{:.1} GB", b / GB)
        } else if b >= MB {
            format!("{:.1} MB", b / MB)
        } else if b >= KB {
            format!("{:.1} KB", b / KB)
        } else {
            format!("{} B", bytes)
        }
    }

    pub fn set_status(&self, msg: &str) {
        self.finish_status();
        *self.current_msg.lock().unwrap() = msg.to_string();

        self.spinner_active.store(true, Ordering::Relaxed);
        print!("\r\x1b[2K{}", msg);
        io::stdout().flush().unwrap();
    }

    pub fn show_progress_count(&self, done: usize, total: usize, msg: &str) {
        self.progress_active.store(true, Ordering::Relaxed);

        let mut first_time = false;
        {
            let mut prog_msg = self.progress_msg.lock().unwrap();
            if *prog_msg != msg {
                *prog_msg = msg.to_string();
                first_time = true;
            }
        }

        if first_time {
            println!("{}", msg);
            io::stdout().flush().unwrap();
        }

        let percent = if total > 0 {
            done as f64 * 100.0 / total as f64
        } else {
            0.0
        };
        let filled = ((done * self.width) / total).min(self.width);
        let empty = self.width - filled;

        let bar = format!(
            "[\x1b[32m{filled_blocks}\x1b[0m{empty_blocks}] {percent:>5.1}%  {done} / {total}",
            filled_blocks = "█".repeat(filled),
            empty_blocks = "░".repeat(empty),
            percent = percent,
            done = done,
            total = total,
        );

        print!("\r\x1b[2K{}", bar);
        io::stdout().flush().unwrap();
    }

    pub fn show_progress_bytes(&self, progress: u64, total: u64, msg: &str, speed: Option<u64>) {
        self.progress_active.store(true, Ordering::Relaxed);

        let mut first_time = false;
        {
            let mut prog_msg = self.progress_msg.lock().unwrap();
            if *prog_msg != msg {
                *prog_msg = msg.to_string();
                first_time = true;
            }
        }

        if first_time {
            println!("{}", msg);
            io::stdout().flush().unwrap();
        }

        let percent = (total != 0)
            .then(|| progress as f64 * 100.0 / total as f64)
            .unwrap_or(0.0);
        let filled = ((progress * self.width as u64) / total) as usize;
        let empty = self.width.saturating_sub(filled);

        let speed_str = speed
            .map(|s| format!("  ({}/s)", CliUi::format_bytes(s)))
            .unwrap_or_default();

        let bar = format!(
            "[\x1b[32m{filled_blocks}\x1b[0m{empty_blocks}] {percent:>5.1}%  {done} / {total}{speed}",
            filled_blocks = "█".repeat(filled),
            empty_blocks = "░".repeat(empty),
            percent = percent,
            done = CliUi::format_bytes(progress as u64),
            total = CliUi::format_bytes(total as u64),
            speed = speed_str
        );

        print!("\r\x1b[2K{}", bar);
        io::stdout().flush().unwrap();
    }

    pub fn finish_progress(&self) {
        self.progress_active.store(false, Ordering::Relaxed);
        println!(" {}", green!("✓"));
        io::stdout().flush().unwrap();
    }

    pub fn finish_status(&self) {
        let prev = self.current_msg.lock().unwrap().clone();
        if !prev.is_empty() {
            self.spinner_active.store(false, Ordering::Relaxed);
            print!("\r\x1b[2K{} {}\n", green!("✓"), prev);
            io::stdout().flush().unwrap();
        }
    }

    // Spinner running in separate thread
    pub fn start_spinner(&self, message: &str) -> SpinnerHandle {
        *self.current_msg.lock().unwrap() = message.to_string();
        self.spinner_active.store(true, Ordering::Relaxed);

        print!("\r\x1b[2K{}", message);
        io::stdout().flush().unwrap();

        let running = Arc::new(AtomicBool::new(true));
        let flag = running.clone();
        let msg_ref = self.current_msg.clone();
        let spinner_active = self.spinner_active.clone();

        let handle = thread::spawn(move || {
            let frames = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
            let mut i = 0;
            while flag.load(Ordering::Relaxed) {
                if !spinner_active.load(Ordering::Relaxed) {
                    thread::sleep(Duration::from_millis(10));
                    continue;
                }

                let frame = frames[i % frames.len()];
                let msg = msg_ref.lock().unwrap().clone();

                if !msg.is_empty() {
                    print!("\r\x1b[2K\x1b[36m{frame}\x1b[0m {msg}");
                    io::stdout().flush().unwrap();
                }

                i += 1;
                thread::sleep(Duration::from_millis(100));
            }
        });

        SpinnerHandle {
            running,
            handle,
            msg_ref: Arc::clone(&self.current_msg),
        }
    }
}

pub struct SpinnerHandle {
    running: Arc<AtomicBool>,
    handle: thread::JoinHandle<()>,
    msg_ref: Arc<Mutex<String>>,
}

impl SpinnerHandle {
    pub fn stop(self) {
        self.running.store(false, Ordering::Relaxed);
        let _ = self.handle.join();

        let msg = self.msg_ref.lock().unwrap().clone();
        if !msg.is_empty() {
            print!("\r\x1b[2K{} {}\n", green!("✓"), msg);
            io::stdout().flush().unwrap();
        }
    }
}
