#[derive(Default)]
pub enum Socket {
    #[default]
    Default,
    Unix(String),
    Tcp(String)
}