use std::sync::Arc;

use anyhow::Result;
use tokio::{sync::mpsc, task::JoinSet};
use tokio_util::sync::CancellationToken;

use crate::{config::AgentConfig, plugins::{Plugin, PluginContext, ServicePlugin, TaskEnvelope}};

pub struct AgentDaemon {
    scheduled: Vec<Box<dyn Plugin>>,
    services: Vec<Box<dyn ServicePlugin>>,
    task_tx: mpsc::Sender<TaskEnvelope>,
    task_rx: mpsc::Receiver<TaskEnvelope>,
    shotdown: CancellationToken,
}

impl AgentDaemon {
    pub fn register_plugin(&mut self, plugin: impl Plugin) {
        self.scheduled.push(Box::new(plugin));
    }

    pub fn register_service(&mut self, service: impl ServicePlugin) {
        self.services.push(Box::new(service));
    }

    pub async fn run(&mut self, config: AgentConfig) -> Result<()> {
        let mut join_set = JoinSet::new();
        let ctx = Arc::new(PluginContext {
            config: config.clone(),
            api_client: self.api_client.clone(),
        });

        Ok(())
    }
}

pub async fn run(_config: AgentConfig) -> Result<()> {
    Ok(())
}
