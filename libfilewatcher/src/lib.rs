#![allow(non_snake_case)]
#![feature(min_specialization)]
extern crate env_logger;
extern crate notify;
#[macro_use]
extern crate phlow;
extern crate phlow_extensions;
extern crate phlow_ffi;

use notify::{Event, EventHandler, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use phlow_extensions::CoreExtensions;
use std::collections::VecDeque;
use std::error::Error;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use string_box::StringBox;
use value_box::ReturnBoxerResult;
use value_box::{ValueBox, ValueBoxPointer};

// Re-export everything from the `value_box_ffi` and `phlow_ffi` in order to tell Rust to include
// the corresponding `no_mangle` functions.
pub use phlow_ffi::*;
pub use value_box_ffi::*;

type AffectedPaths = Vec<PathBuf>;

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
    pub fn information_for(
        _this: &PharoWatcher,
        view: impl phlow::PhlowView,
    ) -> impl phlow::PhlowView {
        view.list()
            .title("Information")
            .items::<PharoWatcher>(|watcher| phlow_all!(vec![("Queue size", watcher.queue_size())]))
            .item_text::<(&str, usize)>(|each| format!("{}: {}", each.0, each.1.to_string()))
            .send::<(&str, usize)>(|each| phlow!(each.1.clone()))
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
    ptr.with_mut(|watcher| {
        path_ptr.with_ref(|path| {
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
    ptr.with_ref_ok(|watcher| match watcher.poll_event() {
        Some(event) => ValueBox::new(event).into_raw(),
        None => std::ptr::null_mut(),
    })
    .unwrap_or_else(|_| std::ptr::null_mut())
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
            .items::<Event>(|event| {
                phlow_all!(vec![
                    ("Event type", phlow!(event.kind.clone())),
                    ("Event paths", phlow!(event.paths.clone())),
                ])
            })
            .item_text::<(&str, &phlow::PhlowObject)>(|each| {
                format!("{}: {}", each.0, each.1.to_string())
            })
            .send::<(&str, &phlow::PhlowObject)>(|each| phlow!(each.1.clone()))
    }
}

#[no_mangle]
pub extern "C" fn filewatcher_event_kind(ptr: *mut ValueBox<Event>) -> *mut ValueBox<EventKind> {
    ptr.with_ref_ok(|event| ValueBox::new(event.kind.clone()).into_raw())
        .unwrap_or_else(|_| std::ptr::null_mut())
}

#[no_mangle]
pub extern "C" fn filewatcher_destroy_event(ptr: *mut ValueBox<Event>) {
    ptr.release();
}

#[no_mangle]
pub extern "C" fn filewatcher_destroy_eventkind(ptr: *mut ValueBox<EventKind>) {
    ptr.release();
}

#[no_mangle]
pub extern "C" fn filewatcher_eventkind_is_any(ptr: *mut ValueBox<EventKind>) -> bool {
    ptr.with_ref_ok(|eventkind| match *eventkind {
        EventKind::Any => true,
        _ => false,
    })
    .unwrap_or(false)
}

#[no_mangle]
pub extern "C" fn filewatcher_eventkind_is_access(ptr: *mut ValueBox<EventKind>) -> bool {
    ptr.with_ref_ok(|eventkind| eventkind.is_access())
        .unwrap_or(false)
}

#[no_mangle]
pub extern "C" fn filewatcher_eventkind_is_create(ptr: *mut ValueBox<EventKind>) -> bool {
    ptr.with_ref_ok(|eventkind| eventkind.is_create())
        .unwrap_or(false)
}

#[no_mangle]
pub extern "C" fn filewatcher_eventkind_is_modify(ptr: *mut ValueBox<EventKind>) -> bool {
    ptr.with_ref_ok(|eventkind| eventkind.is_modify())
        .unwrap_or(false)
}

#[no_mangle]
pub extern "C" fn filewatcher_eventkind_is_remove(ptr: *mut ValueBox<EventKind>) -> bool {
    ptr.with_ref_ok(|eventkind| eventkind.is_remove())
        .unwrap_or(false)
}

#[no_mangle]
pub extern "C" fn filewatcher_eventkind_is_other(ptr: *mut ValueBox<EventKind>) -> bool {
    ptr.with_ref_ok(|eventkind| eventkind.is_other())
        .unwrap_or(false)
}

#[no_mangle]
pub extern "C" fn filewatcher_event_path_size(ptr: *mut ValueBox<Event>) -> usize {
    ptr.with_ref_ok(|event| event.paths.len()).unwrap_or(0)
}

#[no_mangle]
pub extern "C" fn filewatcher_event_path_at(
    ptr: *mut ValueBox<Event>,
    str_ptr: *mut ValueBox<StringBox>,
    index: usize,
) {
    str_ptr
        .with_mut(|contents| {
            ptr.with_ref_ok(|event| {
                contents.set_string(event.paths[index].to_string_lossy().to_string())
            })
        })
        .log();
}

#[phlow::extensions(FileWatcherExtensions, EventKind)]
impl EventKindExtensions {
    #[phlow::view]
    pub fn information_for(
        _this: &EventKind,
        view: impl phlow::PhlowView,
    ) -> impl phlow::PhlowView {
        view.list()
            .title("Information")
            .items::<EventKind>(|event_kind| {
                phlow_all!(vec![
                    ("Is access", event_kind.is_access()),
                    ("Is modify", event_kind.is_modify()),
                    ("Is create", event_kind.is_create()),
                    ("Is remove", event_kind.is_remove()),
                    ("Is other", event_kind.is_other()),
                ])
            })
            .item_text::<(&str, bool)>(|each| format!("{}: {}", each.0, each.1.to_string()))
            .send::<(&str, bool)>(|each| phlow!(each.1.clone()))
    }
}

#[phlow::extensions(FileWatcherExtensions, AffectedPaths)]
impl AffectedPathExtensions {
    #[phlow::view]
    pub fn information_for(
        _this: &AffectedPaths,
        view: impl phlow::PhlowView,
    ) -> impl phlow::PhlowView {
        view.list()
            .title("Information")
            .items::<AffectedPaths>(|affected_paths| phlow_all!(affected_paths.clone()))
            .item_text::<PathBuf>(|each| each.to_string_lossy().to_string())
    }
}

#[phlow::extensions(FileWatcherExtensions, PathBuf)]
impl PathBufExtensions {
    #[phlow::view]
    pub fn information_for(_this: &PathBuf, view: impl phlow::PhlowView) -> impl phlow::PhlowView {
        view.list()
            .title("Information")
            .items::<PathBuf>(|path| phlow_all!(vec![("Path", path.to_string_lossy().to_string())]))
            .item_text::<(&str, &str)>(|each| format!("{}: {}", each.0, each.1.to_string()))
            .send::<(&str, &str)>(|each| phlow!(each.1.clone()))
    }
}
