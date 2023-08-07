use chrono::prelude::*;
use std::io::Write;
use std::thread;
use std::time::Duration;
use std::time::Instant;

use env_logger;

pub fn init() {
    println!("init logger");
    let mut builder = env_logger::Builder::from_default_env();
    builder
        .format(|buf, record| {
            let thread_name = thread::current();
            let name = thread_name.name().unwrap_or("unknown");
            writeln!(
                buf,
                "[{}] {:10.10} | {:10.10}:{:3}| {} {}",
                chrono::Local::now().format("%H:%M:%S.%3f"),
                name,
                record.file().unwrap_or("unknown"),
                record.line().unwrap_or(0),
                record.level(),
                record.args()
            )
        })
        .filter(None, log::LevelFilter::Info)
        .init();
}
