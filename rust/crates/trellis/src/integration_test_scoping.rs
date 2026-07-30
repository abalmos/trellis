//! Integration-test-only contract namespace mapping.

use std::borrow::Cow;

use async_nats::Subject;

/// Immutable namespace applied to one integration-test connection and its contract artifacts.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IntegrationTestScope {
    run_token: String,
    case_token: String,
    logical_prefix: String,
}

/// Invalid integration-test scope or descriptor subject.
#[derive(Debug, thiserror::Error)]
pub enum IntegrationTestScopeError {
    /// A run or case token was not one non-wildcard NATS token.
    #[error("integration test {name} token must be one non-wildcard NATS subject token")]
    InvalidToken {
        /// Token role.
        name: &'static str,
    },
    /// A descriptor subject was not a valid NATS subject.
    #[error("invalid contract descriptor subject `{0}`")]
    InvalidSubject(String),
    /// The descriptor subject was not in a contract-owned subject family.
    #[error("subject `{0}` is not a contract-owned descriptor subject")]
    NotContractOwned(String),
}

impl IntegrationTestScope {
    /// Construct a scope from deterministic run and case tokens.
    pub fn new(
        run_token: impl Into<String>,
        case_token: impl Into<String>,
    ) -> Result<Self, IntegrationTestScopeError> {
        let run_token = run_token.into();
        let case_token = case_token.into();
        validate_token("run", &run_token)?;
        validate_token("case", &case_token)?;
        Ok(Self {
            logical_prefix: format!("It{run_token}{case_token}"),
            run_token,
            case_token,
        })
    }

    /// Return the deterministic run token.
    #[must_use]
    pub fn run_token(&self) -> &str {
        &self.run_token
    }

    /// Return the deterministic case token.
    #[must_use]
    pub fn case_token(&self) -> &str {
        &self.case_token
    }

    /// Scope one contract logical action name before artifact compilation.
    #[must_use]
    pub fn logical_name(&self, name: &str) -> String {
        if name.starts_with(&self.logical_prefix) {
            name.to_string()
        } else {
            format!("{}{name}", self.logical_prefix)
        }
    }

    /// Scope one contract ID while preserving its exact version suffix.
    #[must_use]
    pub fn contract_id(&self, id: &str) -> String {
        let marker = format!("-it-{}-{}", self.run_token, self.case_token);
        if id
            .split_once('@')
            .map_or(id, |(name, _)| name)
            .ends_with(&marker)
        {
            return id.to_string();
        }
        match id.rsplit_once('@') {
            Some((name, version)) => format!("{name}{marker}@{version}"),
            None => format!("{id}{marker}"),
        }
    }

    /// Scope one case-owned identifier without changing its semantic suffix.
    #[must_use]
    pub fn identifier(&self, value: &str) -> String {
        format!("it-{}-{}-{value}", self.run_token, self.case_token)
    }

    /// Scope one contract-owned capability while preserving platform capabilities.
    #[must_use]
    pub fn capability(&self, capability: &str) -> String {
        let Some((namespace, name)) = capability.split_once("::") else {
            return capability.to_string();
        };
        if matches!(
            namespace,
            "trellis.auth"
                | "trellis.core"
                | "trellis.eventlog"
                | "trellis.health"
                | "trellis.jobs"
                | "trellis.state"
        ) {
            return capability.to_string();
        }
        let marker = format!("-it-{}-{}", self.run_token, self.case_token);
        if namespace.ends_with(&marker) {
            capability.to_string()
        } else {
            format!("{namespace}{marker}::{name}")
        }
    }

    /// Resolve a static generated descriptor subject to its scoped concrete subject.
    pub fn descriptor_subject(&self, subject: &str) -> Result<String, IntegrationTestScopeError> {
        Subject::validated(subject)
            .map_err(|_| IntegrationTestScopeError::InvalidSubject(subject.to_string()))?;
        let mut tokens = subject.split('.');
        let family = tokens.next().unwrap_or_default();
        let version = tokens.next().unwrap_or_default();
        let rest = tokens.collect::<Vec<_>>();
        if !matches!(family, "rpc" | "operations" | "events" | "feed" | "feeds") || rest.is_empty()
        {
            return Err(IntegrationTestScopeError::NotContractOwned(
                subject.to_string(),
            ));
        }
        if matches!(
            rest[0],
            "Auth" | "EventLog" | "Health" | "Jobs" | "State" | "Trellis"
        ) {
            return Ok(subject.to_string());
        }
        if rest[0].starts_with(&self.logical_prefix) {
            return Ok(subject.to_string());
        }
        let scoped = format!(
            "{family}.{version}.{}{}.{}",
            self.logical_prefix,
            rest[0],
            rest[1..].join(".")
        )
        .trim_end_matches('.')
        .to_string();
        Subject::validated(&scoped)
            .map_err(|_| IntegrationTestScopeError::InvalidSubject(scoped.clone()))?;
        Ok(scoped)
    }
}

/// Resolve a descriptor subject through an optional immutable connection scope.
pub fn resolve_descriptor_subject<'a>(
    scope: Option<&IntegrationTestScope>,
    subject: &'a str,
) -> Result<Cow<'a, str>, IntegrationTestScopeError> {
    match scope {
        Some(scope) => match scope.descriptor_subject(subject) {
            Ok(subject) => Ok(Cow::Owned(subject)),
            Err(IntegrationTestScopeError::NotContractOwned(_)) => Ok(Cow::Borrowed(subject)),
            Err(error) => Err(error),
        },
        None => Ok(Cow::Borrowed(subject)),
    }
}

fn validate_token(name: &'static str, token: &str) -> Result<(), IntegrationTestScopeError> {
    if token.contains(['.', '*', '>']) || Subject::validated(token).is_err() {
        return Err(IntegrationTestScopeError::InvalidToken { name });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{resolve_descriptor_subject, IntegrationTestScope};

    #[test]
    fn absent_scope_preserves_exact_subject() {
        let subject = "rpc.v1.Entity.Get";
        let resolved = resolve_descriptor_subject(None, subject).unwrap();
        assert!(matches!(resolved, std::borrow::Cow::Borrowed(_)));
        assert_eq!(resolved, subject);
    }

    #[test]
    fn scoped_resolution_preserves_non_contract_subjects() {
        let scope = IntegrationTestScope::new("run1", "case2").unwrap();
        assert_eq!(
            resolve_descriptor_subject(Some(&scope), "health.v1.heartbeat.service.id").unwrap(),
            "health.v1.heartbeat.service.id"
        );
    }

    #[test]
    fn scopes_descriptor_subjects_and_preserves_wildcards() {
        let scope = IntegrationTestScope::new("run1", "case2").unwrap();
        assert_eq!(
            scope.descriptor_subject("rpc.v1.Entity.Get").unwrap(),
            "rpc.v1.Itrun1case2Entity.Get"
        );
        assert_eq!(
            scope
                .descriptor_subject("events.v1.Entity.Changed.*.>")
                .unwrap(),
            "events.v1.Itrun1case2Entity.Changed.*.>"
        );
        assert_eq!(
            scope
                .descriptor_subject("events.v1.Itrun1case2Entity.Changed.*.>")
                .unwrap(),
            "events.v1.Itrun1case2Entity.Changed.*.>"
        );
    }

    #[test]
    fn rejects_internal_and_invalid_subjects() {
        let scope = IntegrationTestScope::new("run1", "case2").unwrap();
        assert!(scope.descriptor_subject("_INBOX.reply").is_err());
        assert!(scope.descriptor_subject("$JS.API.INFO").is_err());
        assert!(scope.descriptor_subject("rpc..broken").is_err());
    }

    #[test]
    fn preserves_platform_descriptor_subjects() {
        let scope = IntegrationTestScope::new("run1", "case2").unwrap();
        assert_eq!(
            scope
                .descriptor_subject("rpc.v1.Auth.Requests.Validate")
                .unwrap(),
            "rpc.v1.Auth.Requests.Validate"
        );
        assert_eq!(
            scope.descriptor_subject("rpc.v1.Jobs.Query").unwrap(),
            "rpc.v1.Jobs.Query"
        );
    }

    #[test]
    fn scopes_contract_capabilities_and_preserves_platform_capabilities() {
        let scope = IntegrationTestScope::new("run1", "case2").unwrap();
        assert_eq!(
            scope.capability("trellis.integration.rpc::read"),
            "trellis.integration.rpc-it-run1-case2::read"
        );
        assert_eq!(
            scope.capability("trellis.auth::admin"),
            "trellis.auth::admin"
        );
    }

    #[test]
    fn preserves_contract_version() {
        let scope = IntegrationTestScope::new("run1", "case2").unwrap();
        assert_eq!(
            scope.contract_id("trellis.integration.rpc-service@v1"),
            "trellis.integration.rpc-service-it-run1-case2@v1"
        );
    }
}
