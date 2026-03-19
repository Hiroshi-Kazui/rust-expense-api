#[derive(Debug, Clone)]
pub enum AppError {
    Network(String),
    Auth(String),
    Server(String),
    BadRequest(String),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::Network(msg) => write!(f, "ネットワークエラー: {}", msg),
            AppError::Auth(msg) => write!(f, "認証エラー: {}", msg),
            AppError::Server(msg) => write!(f, "サーバーエラー: {}", msg),
            AppError::BadRequest(msg) => write!(f, "不正なリクエスト: {}", msg),
        }
    }
}
