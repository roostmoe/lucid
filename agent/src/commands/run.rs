use std::{collections::HashMap, sync::Arc};

use anyhow::Result;
use tokio::{sync::mpsc, task::JoinSet};
use tokio_util::sync::CancellationToken;

use crate::{
    client::ApiClient, config::AgentConfig, plugins::{Plugin, PluginContext, ServicePlugin, TaskEnvelope}
};

pub struct AgentDaemon {
    config: AgentConfig,
    scheduled: Vec<Box<dyn Plugin>>,
    services: Vec<Box<dyn ServicePlugin>>,
    task_tx: mpsc::Sender<TaskEnvelope>,
    task_rx: mpsc::Receiver<TaskEnvelope>,
    shutdown: CancellationToken,
    api_client: ApiClient,
}

impl AgentDaemon {
    pub fn new(config: AgentConfig) -> Result<Self> {
        let (task_tx, task_rx) = mpsc::channel(100);
        Ok(AgentDaemon {
            config: config.clone(),
            scheduled: Vec::new(),
            services: Vec::new(),
            task_tx,
            task_rx,
            shutdown: CancellationToken::new(),
            api_client: ApiClient::new(
                "".into(),
                Some(std::fs::read_to_string(config.auth_key_path())?),
                Some(std::fs::read_to_string(config.auth_cert_path())?),
                Some(std::fs::read_to_string(config.ca_cert_path())?),
            )?,
        })
    }

    pub fn register_plugin(&mut self, plugin: impl Plugin) {
        self.scheduled.push(Box::new(plugin));
    }

    pub fn register_service(&mut self, service: impl ServicePlugin) {
        self.services.push(Box::new(service));
    }

    pub async fn run(mut self) -> Result<()> {
        let mut join_set = JoinSet::new();
        let ctx = Arc::new(PluginContext {
            config: self.config.clone(),
            api_client: self.api_client,
        });

        // drain services into Arc so we can move owned values into tasks
        let services: Vec<Arc<dyn ServicePlugin>> = self.services
            .drain(..)
            .map(|s| Arc::from(s))
            .collect();

        for service in services {
            let task_tx = self.task_tx.clone();
            let shutdown = self.shutdown.clone();
            let ctx = ctx.clone();
            join_set.spawn(async move {
                service.run(&ctx, task_tx, shutdown).await
            });
        }

        for plugin in &self.scheduled {
            let schedule = plugin.schedule().unwrap();
            let task_tx = self.task_tx.clone();
            let plugin_id = plugin.id().to_string();
            let shutdown = self.shutdown.clone();
            join_set.spawn(async move {
                let mut interval = tokio::time::interval(schedule);
                loop {
                    tokio::select! {
                        _ = interval.tick() => {
                            let _ = task_tx.send(TaskEnvelope {
                                plugin_id: plugin_id.clone(),
                                ack_tx: None,
                            }).await;
                        }
                        _ = shutdown.cancelled() => break,
                    }
                }
                Ok(())
            });
        }

        let plugins: Arc<HashMap<String, Box<dyn Plugin>>> = Arc::new(
            self.scheduled.into_iter().map(|p| (p.id().to_string(), p)).collect()
        );

        join_set.spawn(async move {
            while let Some(envelope) = self.task_rx.recv().await {
                if let Some(plugin) = plugins.get(&envelope.plugin_id) {
                    let ctx = ctx.clone();
                    // run in a separate task so plugins don't block the executor
                    let result = plugin.run(&ctx).await;
                    if let Some(ack_tx) = envelope.ack_tx {
                        let _ = ack_tx.send(result.unwrap());
                    }
                }
            }
            Ok(())
        });

        join_set.join_all().await;

        Ok(())
    }
}

pub async fn run(config: AgentConfig) -> Result<()> {
    let daemon = AgentDaemon::new(config)?;

    daemon.run().await?;

    Ok(())
}
