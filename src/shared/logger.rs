use owo_colors::OwoColorize;

pub struct Logger;

impl Logger {
    pub fn info(message: &str) {
        println!("{}", message.blue());
    }

    pub fn activity(message: &str) {
        println!("- {}...", message.bold());
    }

    pub fn success(message: &str) {
        println!("✔ {}", message.green());
    }

    pub fn error(message: &str) {
        println!("- Error: {}", message.red().bold());
    }
}
