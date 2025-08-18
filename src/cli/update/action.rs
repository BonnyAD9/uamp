#[derive(Debug, Copy, Clone, Default)]
pub enum Action {
    #[default]
    Update,
    CheckEnabled,
}
