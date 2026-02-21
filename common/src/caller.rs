use std::sync::Arc;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CallerError {
    #[error("Unauthorized: {reason}")]
    Unauthorized { reason: String },

    #[error("User missing permission: {permission}")]
    Forbidden { permission: String },

    #[error("An unspecified error occurred: {0}")]
    Anyhow(#[from] anyhow::Error),
}

impl CallerError {
    pub fn unauthorized(reason: Option<String>) -> Self {
        Self::Unauthorized {
            reason: reason.unwrap_or_else(|| "No reason provided".to_string()),
        }
    }

    pub fn forbidden(permission: &str) -> Self {
        Self::Forbidden {
            permission: permission.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CallerKind {
    User,
    Agent,

    #[cfg(test)]
    MockCaller,
}

pub trait ApiCaller {
    fn kind(&self) -> CallerKind;
    fn id(&self) -> anyhow::Result<String>;
    fn permissions(&self) -> anyhow::Result<Vec<String>>;
}

pub enum Caller {
    Authenticated(Arc<dyn ApiCaller>),
    Unauthenticated,
}

impl Caller {
    #[tracing::instrument(skip(self))]
    pub fn is_authenticated(&self) -> bool {
        matches!(self, Caller::Authenticated(_))
    }

    #[tracing::instrument(skip(self))]
    pub fn api_caller(&self) -> Result<Arc<dyn ApiCaller>, CallerError> {
        match self {
            Caller::Authenticated(api_caller) => Ok(api_caller.clone()),
            Caller::Unauthenticated => Err(CallerError::unauthorized(Some(
                "Caller is not authenticated".into(),
            ))),
        }
    }

    #[tracing::instrument(skip(self))]
    pub fn is_anonymous(&self) -> bool {
        matches!(self, Caller::Unauthenticated)
    }

    #[tracing::instrument(skip(self))]
    pub async fn can(&self, permission: &str) -> Result<bool, CallerError> {
        match self {
            Caller::Authenticated(api_caller) => {
                Ok(api_caller.permissions()?.contains(&permission.to_string()))
            }
            Caller::Unauthenticated => Ok(false),
        }
    }

    #[tracing::instrument(skip(self))]
    pub async fn require(&self, permission: &str) -> Result<(), CallerError> {
        let can = self.can(permission).await?;
        if !can {
            return Err(CallerError::forbidden(permission));
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use std::fmt::Display;

    use super::*;

    struct MockApiCaller {
        id: String,
        permissions: Vec<String>,
    }
    impl Display for MockApiCaller {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "MockApiCaller({})", self.id)
        }
    }
    impl ApiCaller for MockApiCaller {
        fn kind(&self) -> CallerKind {
            CallerKind::MockCaller
        }

        fn id(&self) -> anyhow::Result<String> {
            Ok(self.id.clone())
        }

        fn permissions(&self) -> anyhow::Result<Vec<String>> {
            Ok(self.permissions.clone())
        }
    }

    #[test]
    fn caller_authenticated_false_when_unauthenticated() {
        let caller = Caller::Unauthenticated;
        assert_eq!(caller.is_authenticated(), false);
    }

    #[test]
    fn caller_anonymous_true_when_unauthenticated() {
        let caller = Caller::Unauthenticated;
        assert_eq!(caller.is_anonymous(), true);
    }

    #[test]
    fn caller_authenticated_true_when_authenticated() {
        let mock_caller = MockApiCaller {
            id: "user123".to_string(),
            permissions: vec!["read".to_string(), "write".to_string()],
        };

        let caller = Caller::Authenticated(Arc::new(mock_caller));
        assert_eq!(caller.is_authenticated(), true);
    }

    #[test]
    fn caller_anonymous_false_when_authenticated() {
        let mock_caller = MockApiCaller {
            id: "user456".to_string(),
            permissions: vec!["read".to_string(), "write".to_string()],
        };

        let caller = Caller::Authenticated(Arc::new(mock_caller));
        assert_eq!(caller.is_anonymous(), false);
    }

    #[tokio::test]
    async fn caller_can_resolves_correctly() {
        let mock_caller = MockApiCaller {
            id: "user456".to_string(),
            permissions: vec!["read".to_string(), "write".to_string()],
        };

        let caller = Caller::Authenticated(Arc::new(mock_caller));

        let can_read = caller.can("read").await.unwrap();
        assert_eq!(can_read, true);

        let can_fire = caller.can("fire").await.unwrap();
        assert_eq!(can_fire, false);
    }

    #[tokio::test]
    async fn caller_require_fails_on_missing_permission() {
        let mock_caller = MockApiCaller {
            id: "user456".to_string(),
            permissions: vec!["write".to_string()],
        };

        let caller = Caller::Authenticated(Arc::new(mock_caller));

        let can_read = caller.require("read").await;
        assert!(can_read.is_err());
    }

    #[tokio::test]
    async fn caller_require_succeeds_on_present_permission() {
        let mock_caller = MockApiCaller {
            id: "user456".to_string(),
            permissions: vec!["write".to_string()],
        };

        let caller = Caller::Authenticated(Arc::new(mock_caller));

        let can_read = caller.require("write").await;
        assert!(!can_read.is_err());
    }
}
