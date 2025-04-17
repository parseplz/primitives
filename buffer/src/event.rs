use crate::Cursor;

// Event's created by the IO
pub enum Event<'a, 'b> {
    Read(&'a mut Cursor<'b>),
    End(&'a mut Cursor<'b>),
}
