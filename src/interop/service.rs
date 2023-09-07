use crate::imports::*;
use crate::result::Result;

pub trait AsyncService : Sync + Send {
    fn start(self: Arc<Self>) -> BoxFuture<'static, Result<()>>;
    fn signal_exit(self: Arc<Self>);
    fn stop(self: Arc<Self>) -> BoxFuture<'static, Result<()>>;
}

