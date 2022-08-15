use std::env;
use notify::{Watcher, watcher, DebouncedEvent};
use std::sync::mpsc::{channel, Receiver};
use std::time::Duration;
use std::fs::read_to_string;
use crate::{Message, parse_message};
use std::process;

pub fn receive() {
    let path = get_socket_file().unwrap_or_else(|| {
        eprintln!("Please provide an argument or the IPCBOX_FILE environment variable");
        process::exit(1);
    });

    let (tx, receiver) = channel();
    let mut watcher = watcher(tx, Duration::from_millis(250)).unwrap();
    watcher.watch(path, notify::RecursiveMode::NonRecursive).unwrap();

    let mut child: Option<process::Child> = None;

    loop {
        let message = receive_message(&receiver);
        execute_message(message, &mut child);
    }
}

fn receive_message(receiver: &Receiver<DebouncedEvent>) -> Option<Message> {
    match receiver.recv() {
        Ok(DebouncedEvent::Write(file)) => {
            let path = file.as_path();
            return parse_message(&read_to_string(path).ok()?)
        },
        _ => None
    }
}

fn execute_message(message: Option<Message>, child: &mut Option<process::Child>) {
    match message {
        None => {},
        Some(Message::Interrupt) => {
            match child {
                Some(c) => {_ = c.kill();},
                _ => {}
            };
        }
        Some(Message::Exit) => {
            match child {
                Some(c) => {_ = c.kill();},
                _ => {}
            };
            std::process::exit(0);
        },
        Some(Message::Command(command)) => {
            match child {
                Some(c) => {_ = c.kill();},
                _ => {}
            };
            match process::Command::new("sh").args(["-c", command.as_str()]).spawn() {
                Ok(c) => {*child = Some(c);},
                _ => {}
            }
        }
    }
}

fn get_socket_file() -> Option<String> {
    return env::args().nth(1).or_else(|| env::var("IPCBOX_FILE").ok())
}
