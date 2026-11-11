use tokio_util::sync::CancellationToken;

#[derive(Debug, Clone)]
pub struct JobCancelHandle {
    token: CancellationToken,
}

impl JobCancelHandle {
    #[must_use]
    pub fn new() -> Self {
        Self {
            token: CancellationToken::new(),
        }
    }

    pub fn cancel(&self) {
        self.token.cancel();
    }

    #[must_use]
    pub fn is_cancelled(&self) -> bool {
        self.token.is_cancelled()
    }

    #[must_use]
    pub fn token(&self) -> CancellationToken {
        self.token.clone()
    }
}

impl Default for JobCancelHandle {
    fn default() -> Self {
        Self::new()
    }
}
