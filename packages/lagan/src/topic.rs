use ntcore_sys::NT_Entry;

use crate::Instance;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NetworkTablesTopic<'a, I: Instance + ?Sized> {
    pub(crate) instance: &'a I,
    pub(crate) handle: NT_Entry,
    pub(crate) name: String,
}