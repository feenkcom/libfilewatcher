#![allow(non_snake_case)]
extern crate notify;

use notify::{PollWatcher, RecursiveMode, Watcher, DebouncedEvent};
use std::sync::mpsc::{channel, Receiver};
use std::time::Duration;
use value_box::{ValueBox, ValueBoxPointer};
use string_box::StringBox;

#[no_mangle]
pub extern "C" fn filewatcher_test() -> bool {
    true
}

#[no_mangle]
pub extern "C" fn filewatcher_create_watcher() -> *mut ValueBox<(PollWatcher, Receiver<DebouncedEvent>)> {
    let (tx, rx) = channel();
    match PollWatcher::new(tx, Duration::from_secs(10)) {
        Ok(watcher) => ValueBox::new((watcher, rx)).into_raw(),
        Err(_) => std::ptr::null_mut()
    }
}

#[no_mangle]
pub extern "C" fn filewatcher_watcher_watch(ptr: *mut ValueBox<(PollWatcher, Receiver<DebouncedEvent>)>, path_ptr: *mut ValueBox<StringBox>) {
    match ptr.to_ref() {
      Ok(mut tuple) =>
        match path_ptr.to_ref() {
            Ok(path) => tuple.0.watch(path.to_string(), RecursiveMode::Recursive).unwrap(),
            Err(_) => ()
        }
      Err(_) => ()
    }
}

#[no_mangle]
pub extern "C" fn filewatcher_destroy_watcher(ptr: *mut ValueBox<(PollWatcher, Receiver<DebouncedEvent>)>) {
    ptr.release();
}

#[no_mangle]
pub extern "C" fn filewatcher_receive_event(ptr: *mut ValueBox<(PollWatcher, Receiver<DebouncedEvent>)>) -> *mut ValueBox<DebouncedEvent> {
    match ptr.to_ref() {
      Ok(tuple) =>
        match tuple.1.recv() {
           Ok(event) => ValueBox::new(event).into_raw(),
           Err(_) => std::ptr::null_mut(),
        }
        Err(_) => std::ptr::null_mut()
    }
}
