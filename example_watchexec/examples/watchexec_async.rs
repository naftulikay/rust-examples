use std::collections::BTreeMap;
use std::ops::AddAssign;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;
use chrono::{DateTime, Utc};
use tokio::sync::Mutex;
use tracing::Level;
use watchexec::action::ActionHandler;
use watchexec::Watchexec;
use watchexec_events::filekind::FileEventKind;
use watchexec_events::Tag;
use watchexec_signals::Signal;

use examples_common::logging::LogLevelFilter;

const CARGO_CRATE_NAME: &str = env!("CARGO_CRATE_NAME");
const CARGO_MANIFEST_DIR: &str = env!("CARGO_MANIFEST_DIR");
const WATCHED_FILE_NAME: &str = "watched";

/// Read/write storage available for use during response to [watchexec] events.
#[derive(Debug, Default)]
struct Runtime {
    /// A map of events that occurred historically
    event_history: BTreeMap<DateTime<Utc>, Vec<ChangeEvent>>,
    /// A count of events that have been processed
    event_count: usize,
}

impl Runtime {
    pub fn new() -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Default::default()))
    }

    /// Process an event that occurred in [watchexec].
    pub async fn on_event(&mut self, mut action: ActionHandler) -> ActionHandler {
        tracing::info!("Received event: {:?}", action);

        let ts = Utc::now();

        // count events
        self.event_count.add_assign(1);

        // note shutdown
        let mut must_exit = false;

        // record event history
        let mut events = Vec::new();

        // process signals first
        for signal in action.signals() {
            events.push(ChangeEvent::SignalReceived(signal));

            if signal == Signal::Interrupt || signal == Signal::Terminate {
                tracing::debug!(ts = ts.to_rfc3339(), %signal, "Received signal to exit");
                must_exit = true;
            } else {
                tracing::debug!(ts = ts.to_rfc3339(), %signal, "Received non-stop signal");
            }
        }

        // then process file events
        for event in action.events.iter() {
            let mut file_path = Option::<PathBuf>::None;
            let mut event_type = Option::<FileChangeKind>::None;

            for tag in event.tags.iter() {
                match tag {
                    Tag::Path { path, .. } => {
                        file_path = Some(path.clone());
                    }
                    Tag::FileEventKind(kind) => {
                        event_type = match kind {
                            FileEventKind::Access(_) => Some(FileChangeKind::Accessed),
                            FileEventKind::Create(_) => Some(FileChangeKind::Created),
                            FileEventKind::Modify(_) => Some(FileChangeKind::Modified),
                            FileEventKind::Remove(_) => Some(FileChangeKind::Removed),
                            _ => None
                        };
                    }
                    _ => {}
                }
            }

            if file_path.is_none() || event_type.is_none() {
                tracing::debug!(ts = ts.to_rfc3339(), "Not a file event, continuing");
                continue;
            } else {
                let (file_path, event_type) = (file_path.unwrap(), event_type.unwrap());

                tracing::debug!(ts = ts.to_rfc3339(), path = %file_path.display(), event_type = ?event_type, "Received file event");

                events.push(ChangeEvent::FileChanged(FileChangeEvent {
                    path: file_path,
                    kind: event_type,
                }));
            }
        }

        // FIXME watchexec also makes it possible to monitor _processes_ and respond to their events

        // record the events
        self.event_history.insert(ts, events);

        // if we need to shut down, make it so
        if must_exit {
            action.quit();
        }

        // return the action
        action
    }
}

#[derive(Debug)]
enum ChangeEvent {
    FileChanged(FileChangeEvent),
    SignalReceived(Signal),
}

#[derive(Debug)]
struct FileChangeEvent {
    #[allow(unused)]
    kind: FileChangeKind,
    #[allow(unused)]
    path: PathBuf,
}

#[derive(Debug)]
enum FileChangeKind {
    Accessed,
    Created,
    Modified,
    Removed,
}

#[tokio::main]
async fn main() -> Result<()> {
    // initialize logging
    examples_common::logging::init_logging(LogLevelFilter::builder()
        .global(Level::WARN)
        .level(CARGO_CRATE_NAME, Level::TRACE)
        .level(examples_common::CRATE_NAME, Level::DEBUG)
        .build());


    // start work
    let workdir = PathBuf::from(CARGO_MANIFEST_DIR);
    let w = workdir.join(WATCHED_FILE_NAME);

    if !w.is_file() {
        // if the file doesn't exist, create it
        tokio::fs::write(&w, "init").await?;
    }

    tracing::info!("Starting watchexec");
    tracing::info!("Modify ./{} to trigger events", PathBuf::from(example_watchexec::CRATE_NAME).join(WATCHED_FILE_NAME).display());

    let rt = Runtime::new();
    let rt2 = rt.clone();

    // NOTE this is the most difficult part of all of this
    /*
        Watchexec::nwe_async accept one argument:
        
            impl Fn(ActionHandler) -> Box<dyn Future<Output=ActionHandler> + Send + Sync> + Send + Sync + 'static
            
        So, what we need here is a regular (non-async) function as the first argument. This function must return a Box co
     */
    let wx = Watchexec::new_async(move |action| {
        let rt = rt2.clone();
        Box::new(async move {
            rt.lock().await.on_event(action).await
        })
    })?;

    wx.config.pathset([w]);
    wx.main().await??;

    let total_events: usize = {
        rt.lock().await.event_history.values().map(|v| v.len()).sum()
    };

    tracing::info!(events = rt.lock().await.event_count, total_events, "Shutting down");

    Ok(())
}