use owo_colors::OwoColorize;

pub struct Logger {}

impl Logger {
    pub fn new() -> Self {
        Logger {}
    }

    pub fn info(&self, message: &str) {
        println!("- {}", message.blue());
    }

    pub fn activity(&self, message: &str) {
        println!("\n- {}...", message.bold());
    }

    pub fn success(&self, message: &str) {
        println!("✔ {}", message.green());
    }

    pub fn error(&self, message: &str) {
        println!("- Error: {}", message.red().bold());
    }
}
