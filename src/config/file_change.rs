use std::{path::Path, thread, time::Duration};
use tokio::sync::mpsc::{Receiver, Sender};

use limero::*;
use log::*;
use notify::{Config, RecommendedWatcher, RecursiveMode, Result, Watcher};

#[derive(Clone, Debug, PartialEq)]
pub enum FileChangeEvent {
    FileChange(String),
}

pub struct FileChangeActor {
    file_name: String,
    events: EventHandlers<FileChangeEvent>,
    cmds: CmdQueue<()>,
    timers: Timers,
}

 impl FileChangeActor {
    pub fn new(file_name: String) -> Self {
        Self {
            file_name,
            events: EventHandlers::new(),
            cmds: CmdQueue::new(1),
            timers: Timers::new(),
        }
    }

    pub fn trigger_file_change(&mut self) {
        self.events.handle(&FileChangeEvent::FileChange("trigger".to_string()));
    }

}

impl Actor<(), FileChangeEvent> for FileChangeActor {
    async fn run(&mut self) {

        let (mut sender,mut receiver ) = tokio::sync::mpsc::channel(10);
        let file_name = self.file_name.clone();
        thread::spawn(move || {
            let _res = watching(file_name,sender);
        });

        info !("FileChange watching {:?}", self.file_name);

        self.timers
            .add_timer(Timer::new_repeater(1, Duration::from_secs(10000)));
        loop {
            tokio::select! {
                m = receiver.recv() => {

                    match m {
                        Some(FileChangeEvent::FileChange(_)) => {
                            info!("FileChange event : {:?}",m );
                            tokio::time::sleep(Duration::from_millis(100)).await;
                            self.events.handle(&FileChangeEvent::FileChange(self.file_name.clone()));
                        }
                        None => {
                            info!("FileChange event None");
                        }
                    }
                }
                _ = self.timers.alarm() => {
                    info!("FileChange timer"    );
                }
            }
        }
    }

    fn add_listener(&mut self, handler: Box<dyn Handler<FileChangeEvent>>) {
        self.events.add_listener(handler);
    }

    fn handler(&self) -> Box<dyn Handler<()>> {
        self.cmds.handler()
    }
}




fn watching( file_name:String , _sender : Sender<FileChangeEvent>) -> notify::Result<()> {
    let path = Path::new(&file_name);
    let (tx, rx) = std::sync::mpsc::channel();
    let mut last_change = std::time::SystemTime::now();

    // Automatically select the best implementation for your platform.
    // You can also access each implementation directly e.g. INotifyWatcher.
    let mut watcher = RecommendedWatcher::new(tx, Config::default())?;

    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.
    watcher.watch(path.as_ref(), RecursiveMode::Recursive)?;

    for res in rx {
        match res {
            Ok(event) => {
                log::info!("changed: {:?}", event);
                let delta = last_change.elapsed().unwrap();
                last_change = std::time::SystemTime::now();
                if delta.as_secs() < 1 {
                    log::info!("delta: {:?}", delta);
                    continue;
                }
                let _ = _sender.try_send(FileChangeEvent::FileChange(path.to_str().unwrap().to_string()));
            },
            Err(error) => log::error!("Error: {error:?}"),
        }
    }

    Ok(())
}