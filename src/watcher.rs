use notify::{
	Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher,
	event::{DataChange, ModifyKind},
};
use std::{
	path::Path,
	sync::{
		Arc,
		atomic::{AtomicBool, Ordering},
	},
	time::Duration,
};
use tokio::{
	sync::{Notify, mpsc},
	time::sleep,
};

pub async fn watch_file(path: &Path, ms: u64, event_tx: mpsc::Sender<Event>) -> notify::Result<()> {
	let notifier = Arc::new(Notify::new());
	let event_holder = Arc::new(tokio::sync::Mutex::new(None));
	let cooldown = Arc::new(AtomicBool::new(false));

	let notifier_cb = notifier.clone();
	let event_holder_cb = event_holder.clone();
	let cooldown_cb = cooldown.clone();

	let mut watcher: RecommendedWatcher = Watcher::new(
		move |res: Result<Event, notify::Error>| {
			if let Ok(event) = res {
				if event.kind == EventKind::Modify(ModifyKind::Data(DataChange::Any))
					&& !cooldown_cb.load(Ordering::Relaxed)
				{
					let mut guard = tokio::task::block_in_place(|| event_holder_cb.blocking_lock());
					*guard = Some(event);
					notifier_cb.notify_one();
				}
			}
		},
		notify::Config::default(),
	)?;

	watcher.watch(path, RecursiveMode::NonRecursive)?;

	let block_duration = Duration::from_millis(ms);

	loop {
		notifier.notified().await;
		cooldown.store(true, Ordering::Relaxed);

		if let Some(ev) = event_holder.lock().await.take() {
			let _ = event_tx.send(ev).await;
		}

		sleep(block_duration).await;
		cooldown.store(false, Ordering::Relaxed);
	}
}
