pub mod client;
pub mod server;
pub mod nt_types;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum NetworkTablesVersion {
    #[default]
    V4,
    V3,
}
