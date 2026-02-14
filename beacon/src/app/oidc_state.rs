use moka::future::Cache;
use std::time::Duration;

/// Cache for OIDC state during login flow (CSRF token → nonce mapping)
pub struct OidcStateCache {
    cache: Cache<String, String>,
}

impl OidcStateCache {
    pub fn new() -> Self {
        let cache = Cache::builder()
            .time_to_live(Duration::from_secs(600)) // 10 minute TTL
            .max_capacity(10_000)
            .build();
        Self { cache }
    }

    pub async fn store(&self, csrf_token: String, nonce: String) {
        self.cache.insert(csrf_token, nonce).await;
    }

    pub async fn get_and_remove(&self, csrf_token: &str) -> Option<String> {
        let nonce = self.cache.get(csrf_token).await;
        if nonce.is_some() {
            self.cache.invalidate(csrf_token).await;
        }
        nonce
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_store_and_retrieve() {
        let cache = OidcStateCache::new();
        let csrf = "csrf-token-123".to_string();
        let nonce = "nonce-456".to_string();

        cache.store(csrf.clone(), nonce.clone()).await;

        let retrieved = cache.cache.get(&csrf).await;
        assert_eq!(retrieved, Some(nonce));
    }

    #[tokio::test]
    async fn test_get_and_remove_removes_entry() {
        let cache = OidcStateCache::new();
        let csrf = "csrf-token-789".to_string();
        let nonce = "nonce-abc".to_string();

        cache.store(csrf.clone(), nonce.clone()).await;

        // First retrieval should succeed
        let retrieved = cache.get_and_remove(&csrf).await;
        assert_eq!(retrieved, Some(nonce));

        // Second retrieval should fail (entry removed)
        let retrieved_again = cache.get_and_remove(&csrf).await;
        assert_eq!(retrieved_again, None);
    }

    #[tokio::test]
    async fn test_get_and_remove_missing_entry() {
        let cache = OidcStateCache::new();
        let result = cache.get_and_remove("non-existent-csrf").await;
        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn test_multiple_entries() {
        let cache = OidcStateCache::new();

        cache.store("csrf1".to_string(), "nonce1".to_string()).await;
        cache.store("csrf2".to_string(), "nonce2".to_string()).await;
        cache.store("csrf3".to_string(), "nonce3".to_string()).await;

        assert_eq!(
            cache.get_and_remove("csrf2").await,
            Some("nonce2".to_string())
        );
        assert_eq!(
            cache.get_and_remove("csrf1").await,
            Some("nonce1".to_string())
        );
        assert_eq!(
            cache.get_and_remove("csrf3").await,
            Some("nonce3".to_string())
        );

        // All should be gone now
        assert_eq!(cache.get_and_remove("csrf1").await, None);
        assert_eq!(cache.get_and_remove("csrf2").await, None);
        assert_eq!(cache.get_and_remove("csrf3").await, None);
    }
}
