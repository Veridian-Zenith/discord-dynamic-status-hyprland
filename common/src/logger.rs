use crate::constants;
use chrono::Local;
use directories::ProjectDirs;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::sync::{Mutex, OnceLock};

pub struct Logger {
    file: Mutex<std::fs::File>,
}

impl Logger {
    fn init(app_name: &str) -> Self {
        let proj_dirs = ProjectDirs::from(constants::QUALIFIER, constants::ORGANIZATION, app_name)
            .expect("Failed to get application directory");

        let log_dir = proj_dirs.data_dir().join("logs");

        if !log_dir.exists() {
            fs::create_dir_all(&log_dir).expect("Failed to create log directory");
        }

        let timestamp = Local::now().format("%Y-%m-%d_%H-%M-%S").to_string();
        let log_path = log_dir.join(format!("{}.log", timestamp));

        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
            .expect("Failed to create log file");

        Logger {
            file: Mutex::new(file),
        }
    }

    pub fn init_logger(app_name: &str) {
        let app_name = app_name.to_string();
        static INSTANCE: OnceLock<Logger> = OnceLock::new();
        INSTANCE.get_or_init(|| Logger::init(&app_name));
    }

    pub fn log(message: &str) {
        static INSTANCE: OnceLock<Logger> = OnceLock::new();
        let instance = INSTANCE.get_or_init(|| Logger::init("Dynamic-DRPC"));

        let now = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
        let log_line = format!("{} - {}\n", now, message);

        print!("{}", log_line);

        let mut file = instance.file.lock().unwrap();

        file.write_all(log_line.as_bytes())
            .expect("Failed to write to log file");
    }
}
