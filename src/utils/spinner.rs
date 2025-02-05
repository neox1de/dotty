use std::io::{self, Write};
use std::sync::mpsc::{self, Sender};
use std::thread;
use std::time::Duration;
use colored::Colorize;

const SPINNER_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

#[derive(Clone)]
pub struct Spinner {
    message: String,
    current_frame: usize,
    stop_sender: Option<Sender<()>>,
}

impl Spinner {
    pub fn new(message: &str) -> Self {
        Self {
            message: message.to_string(),
            current_frame: 0,
            stop_sender: None,
        }
    }

    pub fn tick(&mut self) {
        print!("\r{} {} ", 
            SPINNER_FRAMES[self.current_frame].bright_blue(),
            self.message
        );
        io::stdout().flush().unwrap();
        self.current_frame = (self.current_frame + 1) % SPINNER_FRAMES.len();
    }

    pub fn clear(&self) {
        print!("\r{}\r", " ".repeat(self.message.len() + 2));
        io::stdout().flush().unwrap();
    }
}

pub fn with_spinner<F, T>(message: &str, f: F) -> T
where
    F: FnOnce() -> T,
{
    let mut spinner = Spinner::new(message);
    let (tx, rx) = mpsc::channel();
    spinner.stop_sender = Some(tx);

    let mut spinner_clone = spinner.clone();
    let handle = thread::spawn(move || {
        while rx.try_recv().is_err() {
            spinner_clone.tick();
            thread::sleep(Duration::from_millis(80));
        }
    });

    let result = f();
    
    if let Some(tx) = spinner.stop_sender.take() {
        let _ = tx.send(());
    }
    let _ = handle.join();
    
    spinner.clear();
    result
} 