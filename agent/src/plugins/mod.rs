use std::time::Duration;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc, oneshot};
use tokio_util::sync::CancellationToken;

use crate::{client::ApiClient, config::AgentConfig};

#[derive(Debug)]
pub struct TaskEnvelope {
    pub plugin_id: String,
    /// optional: ack channel for command-triggered tasks
    pub ack_tx: Option<oneshot::Sender<TaskResult>>,
}

pub struct PluginContext {
    pub config: AgentConfig,
    pub api_client: ApiClient,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    pub plugin_id: &'static str,
    pub payload: serde_json::Value,
}

#[async_trait]
pub trait Plugin: Send + Sync + 'static {
    /// Unique identifier for the plugin, used in task definitions and results.
    fn id(&self) -> &'static str;

    /// The schedule method returns an optional Duration indicating when the
    /// plugin should be executed next.
    ///
    /// Plugins that return `None` will not be scheduled for execution, but can
    /// still be triggered manually via the API or CLI.
    ///
    /// Plugins that return `Some(Duration)` will be automatically scheduled to
    /// run after the specified duration has elapsed. After execution, the
    /// schedule method will be called again to determine the next execution
    /// time.
    fn schedule(&self) -> Option<Duration>;

    /// The unit of work to be performed by the plugin. This method will be
    /// called when the plugin is executed, either on a schedule or via manual
    /// trigger.
    ///
    /// The plugin should perform its task and return a TaskResult that will be
    /// sent back to the server.
    async fn run(&self, ctx: &PluginContext) -> anyhow::Result<TaskResult>;
}

#[async_trait]
pub trait ServicePlugin: Send + Send + 'static {
    fn id(&self) -> &'static str;

    async fn run(
        &self,
        ctx: &PluginContext,
        task_tx: mpsc::Sender<TaskEnvelope>,
        mut shutdown: CancellationToken,
    ) -> anyhow::Result<()>;
}
