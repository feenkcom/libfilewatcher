#![allow(non_snake_case)]
extern crate notify;

use notify::{PollWatcher, RecursiveMode, Watcher, DebouncedEvent};
use std::sync::mpsc::{channel, Sender, Receiver};
use std::time::Duration;
use value_box::{ValueBox, ValueBoxPointer};
use string_box::StringBox;


#[no_mangle]
pub fn filewatcher_create_watcher() -> *mut ValueBox<(PollWatcher, Receiver<DebouncedEvent>)> {
    let (tx, rx) = channel();
    match PollWatcher::new(tx, Duration::from_secs(10)) {
        Ok(watcher) => ValueBox::new((watcher, rx)).into_raw(),
        Err(_) => std::ptr::null_mut()
    }
}

#[no_mangle]
pub fn filewatcher_watcher_watch(ptr: *mut ValueBox<PollWatcher>, path_ptr: *mut ValueBox<StringBox>) {
    match ptr.to_ref() {
      Ok(mut watcher) =>
        match path_ptr.to_ref() {
            Ok(path) => watcher.watch(path.to_string(), RecursiveMode::Recursive).unwrap(),
            Err(_) => ()
        }
      Err(_) => ()
    }
}

#[no_mangle]
pub fn filewatcher_destroy_watcher(ptr: *mut ValueBox<PollWatcher>) {
    ptr.release();
}

#[no_mangle]
pub fn filewatcher_destroy_receiver(ptr: *mut ValueBox<Receiver<DebouncedEvent>>) {
    ptr.release();
}

#[no_mangle]
pub fn filewatcher_receive_event(ptr: *mut ValueBox<(Sender<DebouncedEvent>, Receiver<DebouncedEvent>)>) -> *mut ValueBox<DebouncedEvent> {
    match ptr.to_ref() {
      Ok(tuple) =>
        match tuple.1.recv() {
           Ok(event) => ValueBox::new(event).into_raw(),
           Err(_) => std::ptr::null_mut(),
        }
        Err(_) => std::ptr::null_mut()
    }
}
