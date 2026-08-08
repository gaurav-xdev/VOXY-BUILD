use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct CancellationToken {
    id: Uuid,
    cancelled: Arc<AtomicBool>,
}

impl CancellationToken {
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            cancelled: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn with_id(id: Uuid) -> Self {
        Self {
            id,
            cancelled: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Acquire)
    }

    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Release);
    }
}

impl Default for CancellationToken {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct CancellationHandle {
    token: CancellationToken,
}

impl CancellationHandle {
    pub fn new(token: CancellationToken) -> Self {
        Self { token }
    }

    pub fn token(&self) -> &CancellationToken {
        &self.token
    }

    pub fn cancel(&self) {
        self.token.cancel();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_not_cancelled_by_default() {
        let token = CancellationToken::new();
        assert!(!token.is_cancelled());
    }

    #[test]
    fn token_cancellation() {
        let token = CancellationToken::new();
        token.cancel();
        assert!(token.is_cancelled());
    }

    #[test]
    fn cancellation_handle_cancels_token() {
        let token = CancellationToken::new();
        let handle = CancellationHandle::new(token.clone());
        assert!(!token.is_cancelled());
        handle.cancel();
        assert!(token.is_cancelled());
    }

    #[test]
    fn token_with_custom_id() {
        let id = Uuid::new_v4();
        let token = CancellationToken::with_id(id);
        assert_eq!(token.id(), id);
    }

    #[test]
    fn concurrent_cancellation_safety() {
        let token = Arc::new(CancellationToken::new());
        let token_clone = token.clone();
        let handle = std::thread::spawn(move || {
            token_clone.cancel();
        });
        handle.join().unwrap();
        assert!(token.is_cancelled());
    }

    #[test]
    fn token_default_implementation() {
        let token: CancellationToken = Default::default();
        assert!(!token.is_cancelled());
    }

    #[test]
    fn multiple_cancels_idempotent() {
        let token = CancellationToken::new();
        token.cancel();
        token.cancel();
        assert!(token.is_cancelled());
    }

    #[test]
    fn token_equality_by_id() {
        let id = Uuid::new_v4();
        let t1 = CancellationToken::with_id(id);
        let t2 = CancellationToken::with_id(id);
        assert_eq!(t1.id(), t2.id());
    }
}
