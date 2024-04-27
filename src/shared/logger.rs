use owo_colors::OwoColorize;

pub struct Logger {}

impl Logger {
    pub fn new() -> Self {
        Logger {}
    }

    pub fn info(&self, message: &str) {
        println!("- {}", message.blue().bold());
    }

    pub fn activity(&self, message: &str) {
        println!("\n- {}...", message.bold());
    }

    pub fn error(&self, message: &str) {
        println!("- Error: {}", message.to_string().red().bold());
    }
}
