use pareg::FromArg;

#[derive(Debug, PartialEq, Eq, Copy, Clone, FromArg)]
#[arg(exact)]
pub enum EximFormat {
    #[arg("m3u")]
    M3u,
    #[arg("m3u8")]
    M3u8,
    #[arg("pls")]
    Pls,
}

impl EximFormat {
    pub fn from_ext(ext: &str) -> Option<EximFormat> {
        match ext {
            "m3u" => Some(Self::M3u),
            "m3u8" => Some(Self::M3u8),
            "pls" => Some(Self::Pls),
            _ => None,
        }
    }
}
