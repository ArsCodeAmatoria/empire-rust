use crate::core::{Agent, EmpireError, EmpireServer, Task, TaskStatus};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

pub struct Server {
    agents: Arc<RwLock<HashMap<String, Agent>>>,
    tasks: Arc<RwLock<HashMap<String, Task>>>,
}

impl Server {
    pub fn new() -> Self {
        Self {
            agents: Arc::new(RwLock::new(HashMap::new())),
            tasks: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait::async_trait]
impl EmpireServer for Server {
    async fn start(&self) -> Result<(), EmpireError> {
        // TODO: Implement server startup logic
        Ok(())
    }

    async fn stop(&self) -> Result<(), EmpireError> {
        // TODO: Implement server shutdown logic
        Ok(())
    }

    async fn list_agents(&self) -> Result<Vec<Agent>, EmpireError> {
        let agents = self.agents.read().await;
        Ok(agents.values().cloned().collect())
    }

    async fn execute_command(&self, agent_id: &str, command: &str) -> Result<String, EmpireError> {
        let task_id = Uuid::new_v4().to_string();
        let task = Task {
            id: task_id.clone(),
            command: command.to_string(),
            status: TaskStatus::Pending,
            output: None,
            created_at: chrono::Utc::now(),
        };

        {
            let mut tasks = self.tasks.write().await;
            tasks.insert(task_id.clone(), task);
        }

        // TODO: Implement command execution logic
        Ok(task_id)
    }
} 