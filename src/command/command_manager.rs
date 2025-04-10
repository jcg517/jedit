use crate::command::commands::Command;
use crate::document::text_buffer::TextDocument;

pub struct CommandManager {
    //TODO: add undo/redo stacks
}

impl CommandManager {
    pub fn new() -> Self {
        CommandManager {}
    }

    pub fn execute(&mut self, mut command: Box<dyn Command>, data: &mut TextDocument) {
        command.execute(data);

        println!("Executed Command.")
    }
}