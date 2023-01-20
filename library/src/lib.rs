#![allow(non_snake_case)]
#![feature(min_specialization)]
extern crate env_logger;
extern crate notify;
#[macro_use]
extern crate phlow;
extern crate phlow_extensions;

use notify::{Event, EventKind, EventHandler, RecommendedWatcher, RecursiveMode, Watcher};
use phlow_extensions::CoreExtensions;
use std::collections::VecDeque;
use std::error::Error;
use std::path::Path;
use std::sync::{Arc, Mutex};
use string_box::StringBox;
use value_box::ReturnBoxerResult;
use value_box::{ValueBox, ValueBoxPointer};

#[no_mangle]
pub extern "C" fn filewatcher_test() -> bool {
    true
}

#[derive(Debug, Clone, Copy)]
pub struct SemaphoreSignaller {
    semaphore_callback: unsafe extern "C" fn(usize),
    semaphore_index: usize,
}

impl SemaphoreSignaller {
    pub fn new(semaphore_callback: unsafe extern "C" fn(usize), semaphore_index: usize) -> Self {
        Self {
            semaphore_callback,
            semaphore_index,
        }
    }

    pub fn signal(&self) {
        let callback = self.semaphore_callback;
        unsafe { callback(self.semaphore_index) };
    }
}

#[derive(Debug, Clone)]
pub struct FileWatcher {
    signaller: SemaphoreSignaller,
    events: Arc<Mutex<VecDeque<Event>>>,
}

#[no_mangle]
pub fn filewatcher_init_env_logger() {
    env_logger::init();
}

impl FileWatcher {
    pub fn new(
        callback: unsafe extern "C" fn(usize),
        index: usize,
        events: Arc<Mutex<VecDeque<Event>>>,
    ) -> Self {
        Self {
            signaller: SemaphoreSignaller::new(callback, index),
            events,
        }
    }

    pub fn push_event(&self, event: Event) {
        self.events
            .lock()
            .expect("Lock acquisition failed")
            .push_back(event);
        self.signaller.signal()
    }
}

unsafe impl Send for FileWatcher {}

impl EventHandler for FileWatcher {
    fn handle_event(&mut self, event: notify::Result<Event>) {
        if let Ok(event) = event {
            self.push_event(event)
        }
    }
}

#[derive(Debug)]
pub struct PharoWatcher {
    watcher: RecommendedWatcher,
    events: Arc<Mutex<VecDeque<Event>>>,
}

impl PharoWatcher {
    pub fn new(watcher: RecommendedWatcher, events: Arc<Mutex<VecDeque<Event>>>) -> Self {
        Self { watcher, events }
    }

    pub fn watch(&mut self, path: &Path) -> notify::Result<()> {
        self.watcher.watch(path, RecursiveMode::Recursive)
    }

    pub fn poll_event(&self) -> Option<Event> {
        self.events
            .lock()
            .expect("Lock acquisition failed")
            .pop_front()
    }

    pub fn queue_size(&self) -> usize {
        self.events.lock().expect("Lock acquisition failed").len()
    }
}

define_extensions!(FileWatcherExtensions);
import_extensions!(FileWatcherExtensions, CoreExtensions);

#[phlow::extensions(FileWatcherExtensions, PharoWatcher)]
impl WatcherExtensions {
    #[phlow::view]
    pub fn information_for(_this: &PharoWatcher, view: impl phlow::PhlowView) -> impl phlow::PhlowView {
        view.list()
            .title("Information")
            .items(|watcher: &PharoWatcher, _object| {
                phlow_all!(vec![("Queue size", phlow!(watcher.queue_size()))])
            })
            .item_text(|each: &(&str, phlow::PhlowObject), _object| {
                format!("{}: {}", each.0, each.1.to_string())
            })
            .send(|each: &(&str, phlow::PhlowObject), _object| each.1.clone())
    }
}

#[no_mangle]
pub extern "C" fn filewatcher_create_watcher(
    callback: unsafe extern "C" fn(usize),
    index: usize,
) -> *mut ValueBox<PharoWatcher> {
    let events = Arc::new(Mutex::new(VecDeque::new()));
    let watcher = FileWatcher::new(callback, index, events.clone());
    match notify::recommended_watcher(watcher) {
        Ok(notify_watcher) => ValueBox::new(PharoWatcher::new(notify_watcher, events)).into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

#[no_mangle]
pub extern "C" fn filewatcher_watcher_watch(
    ptr: *mut ValueBox<PharoWatcher>,
    path_ptr: *mut ValueBox<StringBox>,
) {
    ptr.to_ref()
        .and_then(|mut watcher| {
            path_ptr.to_ref().and_then(|path| {
                watcher
                    .watch(Path::new(&path.to_string()))
                    .map_err(|error| (Box::new(error) as Box<dyn Error>).into())
            })
        })
        .log();
}

#[no_mangle]
pub extern "C" fn filewatcher_watcher_poll(
    ptr: *mut ValueBox<PharoWatcher>,
) -> *mut ValueBox<Event> {
    match ptr.to_ref() {
        Ok(watcher) =>
            match watcher.poll_event() {
                Some(event) => ValueBox::new(event).into_raw(),
                None => std::ptr::null_mut()
            }
        Err(_) => std::ptr::null_mut()
    }
}

#[no_mangle]
pub extern "C" fn filewatcher_destroy_watcher(ptr: *mut ValueBox<PharoWatcher>) {
    ptr.release();
}

#[phlow::extensions(FileWatcherExtensions, Event)]
impl EventExtensions {
    #[phlow::view]
    pub fn information_for(_this: &Event, view: impl phlow::PhlowView) -> impl phlow::PhlowView {
        view.list()
            .title("Information")
            .items(|event: &Event, _object| {
                phlow_all!(vec![
                    ("Event type", phlow!(event.kind.clone())),
                    ("Event paths", phlow!(event.paths.clone())),
                ])
            })
            .item_text(|each: &(&str, phlow::PhlowObject), _object| {
                format!("{}: {}", each.0, each.1.to_string())
            })
            .send(|each: &(&str, phlow::PhlowObject), _object| each.1.clone())
    }
}

#[no_mangle]
pub extern "C" fn filewatcher_event_kind(
    ptr: *mut ValueBox<Event>,
) -> *mut ValueBox<EventKind> {
    match ptr.to_ref() {
        Ok(event) => ValueBox::new(event.kind.clone()).into_raw(),
        Err(_) => std::ptr::null_mut()
    }
}

#[no_mangle]
pub extern "C" fn filewatcher_eventkind_is_any(
    ptr: *mut ValueBox<EventKind>
) -> bool {
    match ptr.to_ref() {
        Ok(eventkind) =>
            match *eventkind {
                EventKind::Any => true,
                _ => false
            }
        Err(_) => false
    }
}

#[no_mangle]
pub extern "C" fn filewatcher_eventkind_is_access(
    ptr: *mut ValueBox<EventKind>
) -> bool {
    match ptr.to_ref() {
        Ok(eventkind) => eventkind.is_access(),
        Err(_) => false
    }
}

#[no_mangle]
pub extern "C" fn filewatcher_eventkind_is_create(
    ptr: *mut ValueBox<EventKind>
) -> bool {
    match ptr.to_ref() {
        Ok(eventkind) => eventkind.is_create(),
        Err(_) => false
    }
}

#[no_mangle]
pub extern "C" fn filewatcher_eventkind_is_modify(
    ptr: *mut ValueBox<EventKind>
) -> bool {
    match ptr.to_ref() {
        Ok(eventkind) => eventkind.is_modify(),
        Err(_) => false
    }
}

#[no_mangle]
pub extern "C" fn filewatcher_eventkind_is_remove(
    ptr: *mut ValueBox<EventKind>
) -> bool {
    match ptr.to_ref() {
        Ok(eventkind) => eventkind.is_remove(),
        Err(_) => false
    }
}

#[no_mangle]
pub extern "C" fn filewatcher_eventkind_is_other(
    ptr: *mut ValueBox<EventKind>
) -> bool {
    match ptr.to_ref() {
        Ok(eventkind) => eventkind.is_other(),
        Err(_) => false
    }
}

#[no_mangle]
pub extern "C" fn filewatcher_event_path_size(
    ptr: *mut ValueBox<Event>,
) -> *mut ValueBox<usize> {
    match ptr.to_ref() {
        Ok(event) => ValueBox::new(event.paths.len()).into_raw(),
        Err(_) => std::ptr::null_mut()
    }
}

#[no_mangle]
pub extern "C" fn filewatcher_event_path_at(
    ptr: *mut ValueBox<Event>,
    str_ptr: *mut ValueBox<StringBox>,
    index: usize,
) {
    match ptr.to_ref() {
        Ok(event) =>
            match str_ptr.to_ref() {
                Ok(mut contents) =>
                    contents.set_string(event.paths[index].to_string_lossy().to_string()),
                Err(_) => ()
            }
        Err(_) => ()
    }
}
