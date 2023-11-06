use std::path::PathBuf;

use crate::imports::*;
// use kaspa_core::core::Core;
// use kaspa_core::signals::Shutdown;
// use kaspa_rpc_service::service::RpcCoreService;
// use kaspa_utils::fd_budget;
// use kaspa_wallet_core::rpc::DynRpcApi;
// use kaspad_lib::args::Args;
use crate::runtime::kaspa::Config;
// use kaspad_lib::daemon::{create_core_with_runtime, Runtime as KaspadRuntime};

// use std::future::Future;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

#[derive(Default)]
struct Inner {
    path: Option<PathBuf>,
    // child: Option<tokio::process::Child>,
    pid: Option<u32>,
}

#[derive(Default, Clone)]
pub struct Daemon {
    inner: Arc<Mutex<Inner>>,
}

impl Daemon {
    pub fn new(path: PathBuf) -> Self {
        Self {
            inner: Arc::new(Mutex::new(Inner {
                path: Some(path),
                // child: None,
                pid: None,
            })),
        }
    }

    fn inner(&self) -> MutexGuard<'_, Inner> {
        self.inner.lock().unwrap()
    }
    // pub fn rpc_core_services(&self) -> Option<Arc<DynRpcApi>> {
    //     if let Some(inner) = self.inner.lock().unwrap().as_ref() {
    //         inner.rpc_core_service.clone()
    //     } else {
    //         None
    //     }
    // }

    // create_args(args : Args) -> Vec<String> {
    //     let mut list = Vec::new();

    // }
}

#[async_trait]
impl super::Kaspad for Daemon {
    async fn start(&self, config: Config) -> Result<()> {
        // println!("ARGS: {:#?}", args);

        // let mut cmd_args: Vec<String> = Vec::new();

        let mut cmd = if let Some(path) = self.inner().path.clone() {
            Command::new(path)
        } else {
            let path = std::env::current_exe()?;
            Command::new(path)
        };

        cmd.env("KASPA_NG_NODE", "1")
            .args(config)
            .stdout(Stdio::piped());

        let mut child = cmd.spawn().map_err(Error::NodeStartupError)?;
        // .expect("failed to spawn command");

        let stdout = child.stdout.take().ok_or(Error::NodeStdoutHandleError)?;

        self.inner.lock().unwrap().pid = child.id();

        let mut reader = BufReader::new(stdout).lines();

        tokio::spawn(async move {
            // let child = this.inner.lock().unwrap().child.clone().unwrap();
            let status = child
                .wait()
                .await
                .expect("child process encountered an error");

            println!("child status was: {}", status);

            loop {
                println!("loop...");
                select! {
                    status = child.wait().fuse() => {
                        match status {
                            Ok(status) => {
                                println!("child status was: {:?}", status);
                            }
                            Err(err) => {
                                println!("child error was: {:?}", err);
                            }
                        }
                        break;
                    }

                    line = reader.next_line().fuse() => {
                        if let Ok(Some(line)) = line {
                            println!("Line: {}", line);
                        }

                        // match line {
                        //     Ok(Some(line)) => {
                        //         println!("Line: {}", line);
                        //     }
                        //     Err(err) => {
                        //         println!("Line error: {:?}", err);
                        //     }
                        // }
                    }
                }
            }
        });

        // while let Some(line) = reader.next_line().await? {
        //     println!("Line: {}", line);
        // }

        // self.inner.lock().unwrap().replace(Inner {
        //     path : None,
        // });

        println!("***");

        Ok(())
    }

    async fn stop(&self) -> Result<()> {
        // if let Some(mut inner) = self.inner.lock().unwrap().take() {

        //     println!("***");
        // }
        Ok(())
    }
}
