mod receiver;

#[derive(Debug)]
pub enum Message{
    Command(String),
    Interrupt,
    Exit,
}

fn parse_message(input: &str) -> Option<Message> {
    if input.len() < 4 || &input[(input.len() - 4)..] != "\nOK\n" {
        return None
    }
    return match &input[0..(input.len() - 4)] {
        "INTERRUPT" => Some(Message::Interrupt),
        "EXIT" => Some(Message::Exit),
        command => Some(Message::Command(command.to_string()))
    }
}

fn main() {
    receiver::receive();
}

