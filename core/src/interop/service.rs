use crate::imports::*;
use crate::result::Result;

#[async_trait]
pub trait Service: Sync + Send {
    async fn spawn(self: Arc<Self>) -> Result<()>;
    async fn join(self: Arc<Self>) -> Result<()>;
    fn terminate(self: Arc<Self>);
}
