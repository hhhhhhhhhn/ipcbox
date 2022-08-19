use std::env;
use std::io::Write;
use notify::{Watcher, watcher, DebouncedEvent};
use std::sync::mpsc::{channel, Receiver};
use std::time::Duration;
use std::fs::read_to_string;
use crate::{Message, parse_message};
use std::process;

pub fn receive() {
    let path = get_socket_file()
        .expect("Please provide an argument or the IPCBOX_FILE environment variable");

    let (tx, receiver) = channel();
    let mut watcher = watcher(tx, Duration::from_millis(250)).unwrap();
    watcher.watch(path, notify::RecursiveMode::NonRecursive).unwrap();

    let mut child = process::Command::new("sh")
        .args(["-i"])   // Interative mode allows for interrupting the process
                        // without killing the shell.
        .env("PS1", "") // Removes the prompt
        .stdin(process::Stdio::piped())
        .spawn()
        .expect("Could not spawn shell");

    let mut stdin = child.stdin.take().expect("Could not open stdin");

    loop {
        let message = receive_message(&receiver);
        execute_message(message, &mut child, &mut stdin);
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

fn execute_message(message: Option<Message>, child: &mut process::Child, stdin: &mut process::ChildStdin) {
    match message {
        None => {},
        Some(Message::Interrupt) => {
            process::Command::new("kill")
                .args(["-INT", &child.id().to_string()])
                .spawn()
                .expect("Could not spawn interrupt signal");
        }
        Some(Message::Exit) => {
            child.kill().expect("Could not kill child");
            std::process::exit(0);
        },
        Some(Message::Command(command)) => {
            stdin.write_all(command.as_bytes()).expect("Could not write message");
        }
    }
}

fn get_socket_file() -> Option<String> {
    return env::args().nth(1).or_else(|| env::var("IPCBOX_FILE").ok())
}
