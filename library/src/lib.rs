#![allow(non_snake_case)]
extern crate notify;

use notify::{Watcher, RecursiveMode, watcher};
use std::sync::mpsc::channel;
use std::time::Duration;
use string_box::StringBox;
use value_box::{BoxerError, ReturnBoxerResult, ValueBox, ValueBoxPointer};

#[no_mangle]
pub fn filewatcher_create_channel() -> *mut ValueBox<(Sender<DebouncedEvent>, Receiver<DebouncedEvent>)> {
    ValueBox::new(channel()).into_raw()
}

#[no_mangle]
pub fn filewatcher_destroy_channel(ptr: *mut ValueBox<(Sender<DebouncedEvent>, Receiver<DebouncedEvent>)>) {
    ptr.release();
}

#[no_mangle]
pub fn filewatcher_create_watcher(ptr: *mut ValueBox<(Sender<DebouncedEvent>, Receiver<DebouncedEvent>)>) -> *mut ValueBox<Watcher> {
    ptr.to_ref().and_then(|tuple|
        match watcher(tuple.0, RecursiveMode::Recursive) {
            Ok(watcher) => ValueBox::new(watcher).into_raw(),
            Err(_) => std::ptr::null_mut(),
        }).log()
}

#[no_mangle]
pub fn filewatcher_destroy_watcher(ptr: *mut ValueBox<Watcher>) {
    ptr.release();
}

#[no_mangle]
pub fn filewatcher_receive_event(ptr: *mut ValueBox<(Sender<DebouncedEvent>, Receiver<DebouncedEvent>)>) -> *mut ValueBox<DebouncedEvent> {
    ptr.to_ref().and_then(|tuple|
        match tuple.1.recv() {
           Ok(event) => ValueBox::new(event).into_raw(),
           Err(_) => std::ptr::null_mut(),
        }
    ).log()
}
