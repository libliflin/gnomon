use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Violation {
    pub kind: ViolationKind,
    pub metric: &'static str,
    pub budget: u64,
    pub actual: u64,
    pub detail: String,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ViolationKind {
    /// A byte budget was exceeded (HTML, CSS, JS, etc.).
    Bytes,
    /// A count budget was exceeded (requests, third-party domains, etc.).
    Count,
    /// A forbidden domain or script was loaded.
    Forbidden,
    /// Anti-theater rule hit: lazy LCP, preload-as-stylesheet trick, etc.
    Theater,
    /// A fetched resource returned a non-2xx status or timed out.
    FetchError,
}

impl ViolationKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Bytes => "bytes",
            Self::Count => "count",
            Self::Forbidden => "forbidden",
            Self::Theater => "theater",
            Self::FetchError => "fetch",
        }
    }
}
