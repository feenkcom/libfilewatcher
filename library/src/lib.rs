#![allow(non_snake_case)]
#![feature(result_option_inspect)]
extern crate notify;
extern crate env_logger;

use std::error::Error;
use notify::{PollWatcher, RecursiveMode, Watcher, DebouncedEvent};
use std::sync::mpsc::{channel, Receiver};
use std::time::Duration;
use value_box::{ValueBox, ValueBoxPointer};
use string_box::StringBox;
use value_box::ReturnBoxerResult;

#[no_mangle]
pub extern "C" fn filewatcher_test() -> bool {
    true
}

#[no_mangle]
pub fn filewatcher_init_env_logger() {
    env_logger::init();
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
    ptr.to_ref()
        .and_then(|mut tuple| {
            path_ptr.to_ref().and_then(|path| {
                tuple
                    .0
                    .watch(path.to_string(), RecursiveMode::Recursive)
                    .map_err(|error| (Box::new(error) as Box<dyn Error>).into())
            })
        })
        .log();
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

#[no_mangle]
pub extern "C" fn filewatcher_destroy_event(ptr: *mut ValueBox<DebouncedEvent>) {
    ptr.release();
}

#[no_mangle]
pub extern "C" fn filewatcher_event_path(ptr: *mut ValueBox<DebouncedEvent>, contents_ptr: *mut ValueBox<StringBox>) {
    match ptr.to_ref() {
      Ok(event) =>
        match contents_ptr.to_ref() {
            Ok(mut contents) =>
                match &*event {
                    DebouncedEvent::NoticeWrite(path_buf) => {
                        path_buf.to_str().inspect(|s| contents.set_string(s.to_string()));
                        ()
                    },
                    DebouncedEvent::NoticeRemove(path_buf) => {
                        path_buf.to_str().inspect(|s| contents.set_string(s.to_string()));
                        ()
                    },
                    DebouncedEvent::Create(path_buf) => {
                        path_buf.to_str().inspect(|s| contents.set_string(s.to_string()));
                        ()
                    },
                    DebouncedEvent::Write(path_buf) => {
                        path_buf.to_str().inspect(|s| contents.set_string(s.to_string()));
                        ()
                    },
                    DebouncedEvent::Chmod(path_buf) => {
                        path_buf.to_str().inspect(|s| contents.set_string(s.to_string()));
                        ()
                    },
                    DebouncedEvent::Remove(path_buf) => {
                        path_buf.to_str().inspect(|s| contents.set_string(s.to_string()));
                        ()
                    },
                    DebouncedEvent::Rename(_, path_buf) => {
                        path_buf.to_str().inspect(|s| contents.set_string(s.to_string()));
                        ()
                    },
                    DebouncedEvent::Rescan => (),
                    DebouncedEvent::Error(_, _) => (),
                },
            Err(_) => ()
        },
        Err(_) => ()
    }
}

#[no_mangle]
pub extern "C" fn filewatcher_event_type(ptr: *mut ValueBox<DebouncedEvent>, contents_ptr: *mut ValueBox<StringBox>) {
    match ptr.to_ref() {
      Ok(event) =>
        match contents_ptr.to_ref() {
            Ok(mut contents) =>
                match &*event {
                    DebouncedEvent::NoticeWrite(_) =>
                        contents.set_string("notice_write".to_string()),
                    DebouncedEvent::NoticeRemove(_) =>
                        contents.set_string("notice_remove".to_string()),
                    DebouncedEvent::Create(_) =>
                        contents.set_string("create".to_string()),
                    DebouncedEvent::Write(_) =>
                        contents.set_string("write".to_string()),
                    DebouncedEvent::Chmod(_) =>
                        contents.set_string("chmod".to_string()),
                    DebouncedEvent::Remove(_) =>
                        contents.set_string("remove".to_string()),
                    DebouncedEvent::Rename(_, _) =>
                        contents.set_string("rename".to_string()),
                    DebouncedEvent::Rescan =>
                        contents.set_string("rescan".to_string()),
                    DebouncedEvent::Error(_, _) => (),
                },
            Err(_) => ()
        },
        Err(_) => ()
    }
}
