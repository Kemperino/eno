use std::collections::HashMap;

use crate::session::SessionState;

/// Base port for resource allocation
pub const BASE_PORT: u16 = 9100;
/// Port range per agent
pub const PORT_RANGE: u16 = 100;

/// Resource coordinator for managing agent-specific resource allocations
#[derive(Default)]
pub struct ResourceCoordinator;

impl ResourceCoordinator {
    pub fn new() -> Self {
        Self
    }

    /// Calculate the port base for a specific agent
    pub fn port_base_for_agent(&self, agent_id: usize) -> u16 {
        BASE_PORT + ((agent_id - 1) as u16) * PORT_RANGE
    }

    /// Generate environment variables for a specific agent
    pub fn env_for_agent(&self, agent_id: usize, session: &SessionState) -> HashMap<String, String> {
        let port_base = self.port_base_for_agent(agent_id);

        let mut env = HashMap::new();

        // Agent identification
        env.insert("ENO_AGENT_ID".to_string(), agent_id.to_string());
        env.insert("ENO_AGENT_COUNT".to_string(), session.agents.len().to_string());
        env.insert("ENO_SESSION_ID".to_string(), session.id.clone());

        // Port isolation
        env.insert("ENO_PORT_BASE".to_string(), port_base.to_string());
        env.insert("ENO_PORT_RANGE".to_string(), PORT_RANGE.to_string());

        // Docker isolation
        env.insert("ENO_DOCKER_PREFIX".to_string(), format!("eno-{}-", agent_id));
        env.insert("COMPOSE_PROJECT_NAME".to_string(), format!("eno-agent-{}", agent_id));

        // State directory
        env.insert("ENO_STATE_DIR".to_string(), session.state_dir.display().to_string());

        // Convenience variables for common ports
        env.insert("ENO_HTTP_PORT".to_string(), port_base.to_string());
        env.insert("ENO_HTTPS_PORT".to_string(), (port_base + 1).to_string());
        env.insert("ENO_DB_PORT".to_string(), (port_base + 32).to_string());
        env.insert("ENO_REDIS_PORT".to_string(), (port_base + 79).to_string());

        env
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_port_allocation() {
        let coordinator = ResourceCoordinator::new();

        assert_eq!(coordinator.port_base_for_agent(1), 9100);
        assert_eq!(coordinator.port_base_for_agent(2), 9200);
        assert_eq!(coordinator.port_base_for_agent(3), 9300);
        assert_eq!(coordinator.port_base_for_agent(4), 9400);
    }
}
