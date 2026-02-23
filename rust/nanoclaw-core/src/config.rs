#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeConfig {
    pub assistant_name: String,
    pub poll_interval_ms: u64,
    pub idle_timeout_ms: u64,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            assistant_name: "Andy".to_string(),
            poll_interval_ms: 3000,
            idle_timeout_ms: 180_000,
        }
    }
}

impl RuntimeConfig {
    pub fn from_env() -> Self {
        let default = Self::default();

        let assistant_name = std::env::var("ASSISTANT_NAME").unwrap_or(default.assistant_name);

        let poll_interval_ms = std::env::var("POLL_INTERVAL_MS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(default.poll_interval_ms);

        let idle_timeout_ms = std::env::var("IDLE_TIMEOUT_MS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(default.idle_timeout_ms);

        Self {
            assistant_name,
            poll_interval_ms,
            idle_timeout_ms,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::RuntimeConfig;

    #[test]
    fn loads_defaults_when_env_missing() {
        let cfg = RuntimeConfig::from_env();
        assert!(!cfg.assistant_name.is_empty());
        assert!(cfg.poll_interval_ms > 0);
        assert!(cfg.idle_timeout_ms > 0);
    }
}
