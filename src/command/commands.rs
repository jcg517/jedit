use crate::document::text_document::TextDocument;

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
    fn execute(&self, data: &mut TextDocument) {
        todo!();
    }
}

pub struct DeleteCommand {
    pub pos: usize,
    pub len: usize,
}

impl DeleteCommand {
    pub fn new(pos: usize, len: usize) -> Self {
        DeleteCommand { pos, len }
    }
}
impl Command for DeleteCommand {
    fn execute(&self, data: &mut TextDocument) {
        todo!();
    }
}