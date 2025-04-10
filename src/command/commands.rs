use crate::document::text_buffer::TextDocument;

pub trait Command {
    pub fn execute(&self, data: &mut TextDocument);

    //TODO: add undo method for all commands
}

pub struct InsertCommand {
    pub pos: usize,
    pub text: String,
}

impl InsertCommand {
    pub fn new(pos: usize, text: String) -> Self {
        InsertCommand { pos, text }
    }
}

impl Command for InsertCommand {
    fn execute(&mut self, data: &mut TextDocument) {
        todo!();
        println!("InsertCommand executed.");
    }
}

pub struct DeleteCommand {
    pub pos: usize,
    pub len: usize,
}

impl DeleteCommand {
    pub fn new(&mut self, pos: usize, len: usize) -> Self {
        DeleteCommand { pos, len }
    }
}
impl Command for DeleteCommand {
    fn execute(&mut self, data: &mut TextDocument) {
        todo!();
        println!("DeleteCommand executed.")
    }
}