use crate::imports::*;
use crate::result::Result;

#[async_trait]
pub trait Service: Sync + Send {
    async fn spawn(self: Arc<Self>) -> Result<()>;
    async fn join(self: Arc<Self>) -> Result<()>;
    // fn start_service(self: Arc<Self>) -> BoxFuture<'static, Result<()>>;
    // fn stop_service(self: Arc<Self>) -> BoxFuture<'static, Result<()>>;
    fn terminate(self: Arc<Self>);
}

