use crate::constants;
use chrono::Local;
use directories::ProjectDirs;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::sync::{Mutex, OnceLock};

static INSTANCE: OnceLock<Logger> = OnceLock::new();

pub struct Logger {
    file: Mutex<std::fs::File>,
}

impl Logger {
    fn init(app_name: &str) -> Self {
        let proj_dirs = ProjectDirs::from(constants::QUALIFIER, constants::ORGANIZATION, app_name)
            .expect("Failed to determine application directories");

        let log_dir = proj_dirs.data_dir().join("logs");

        fs::create_dir_all(&log_dir).expect("Failed to create log directory");

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
        INSTANCE.get_or_init(|| Logger::init(&app_name));
    }

    pub fn log(message: &str) {
        let instance = INSTANCE.get_or_init(|| Logger::init("Dynamic-DRPC"));

        let now = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
        let log_line = format!("{} - {}\n", now, message);

        print!("{}", log_line);

        if let Ok(mut file) = instance.file.lock() {
            let _ = file.write_all(log_line.as_bytes());
        }
    }
}
