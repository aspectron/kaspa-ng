use crate::imports::*;
use crate::runtime::services::kaspa::{Config, KaspadServiceEvents};
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use workflow_core::prelude::DuplexChannel;

/// Termination method with which to terminate the kaspad process.
/// This should remain Sigkill until Kaspad learns to terminate
/// rapidly during it's sync process.
#[derive(Default, Debug, Clone, Eq, PartialEq)]
enum TerminationMethod {
    #[default]
    Sigkill,
    Sigterm,
}

struct Inner {
    path: Option<PathBuf>,
    is_running: Arc<AtomicBool>,
    pid: Mutex<Option<u32>>,
    service_events: Channel<KaspadServiceEvents>,
    task_ctl: DuplexChannel,
    termination_method: TerminationMethod,
}

#[derive(Clone)]
pub struct Daemon {
    inner: Arc<Inner>,
}

impl Daemon {
    pub fn new(path: Option<PathBuf>, service_events: &Channel<KaspadServiceEvents>) -> Self {
        Self {
            inner: Arc::new(Inner {
                path,
                is_running: Arc::new(AtomicBool::new(false)),
                pid: Mutex::new(None),
                service_events: (*service_events).clone(),
                task_ctl: DuplexChannel::oneshot(),
                termination_method: TerminationMethod::default(),
            }),
        }
    }

    fn inner(&self) -> &Inner {
        &self.inner
    }

    fn is_running(&self) -> bool {
        self.inner().is_running.load(Ordering::SeqCst)
    }

    #[cfg(unix)]
    fn sigterm(&self, pid: u32) {
        use nix::sys::signal::Signal;
        use nix::unistd::Pid;
        if let Err(err) = nix::sys::signal::kill(Pid::from_raw(pid as i32), Signal::SIGTERM) {
            println!("kaspad sigterm error: {:?}", err);
        }
    }
}

#[async_trait]
impl super::Kaspad for Daemon {
    async fn start(self: Arc<Self>, config: Config) -> Result<()> {
        let mut cmd = if let Some(path) = self.inner().path.clone() {
            Command::new(path)
        } else {
            let path = std::env::current_exe()?;
            Command::new(path)
        };

        let cmd = cmd
            .args(config)
            .env("KASPA_NG_DAEMON", "1")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let is_running = self.inner().is_running.clone();
        is_running.store(true, Ordering::SeqCst);
        let mut child = cmd.spawn().map_err(Error::NodeStartupError)?;
        let stdout = child.stdout.take().ok_or(Error::NodeStdoutHandleError)?;
        let stderr = child.stderr.take().ok_or(Error::NodeStdErrHandleError)?;
        *self.inner.pid.lock().unwrap() = child.id();

        let stdout_relay_sender = self.inner.service_events.sender.clone();
        tokio::spawn(async move {
            let mut reader = BufReader::new(stdout).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                let _ = stdout_relay_sender
                    .send(KaspadServiceEvents::Stdout { line })
                    .await;
            }
        });

        let stderr_relay_sender = self.inner.service_events.sender.clone();
        tokio::spawn(async move {
            let mut reader = BufReader::new(stderr).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                stderr_relay_sender
                    .send(KaspadServiceEvents::Stderr { line })
                    .await
                    .unwrap();
            }
        });

        let task_ctl = self.inner.task_ctl.clone();

        let this = self.clone();

        cfg_if::cfg_if! {
            if #[cfg(unix)] {
                let is_unix = true;
            } else {
                let is_unix = false;
            }
        }

        tokio::spawn(async move {
            loop {
                select! {
                    _ = task_ctl.request.recv().fuse() => {
                        if this.inner.termination_method == TerminationMethod::Sigterm && is_unix {
                            let pid = this.inner.pid.lock().unwrap();
                            if let Some(_pid) = *pid {
                                #[cfg(unix)]
                                this.sigterm(_pid);
                            }
                        } else if let Err(err) = child.start_kill() {
                            println!("kaspa daemon start_kill error: {:?}", err);
                        }
                    }
                    status = child.wait().fuse() => {
                        match status {
                            Ok(status) => {
                                if !status.success() {
                                    this.inner.service_events.sender
                                        .send(KaspadServiceEvents::Stderr {
                                            line: format!("kaspad exited with {status}"),
                                        })
                                        .await
                                        .unwrap();
                                }
                            }
                            Err(err) => {
                                println!("kaspad shutdown error: {:?}", err);
                            }
                        }
                        is_running.store(false,Ordering::SeqCst);
                        break;
                    }
                }
            }

            let _ = task_ctl.response.send(()).await;
        });

        Ok(())
    }

    async fn stop(self: Arc<Self>) -> Result<()> {
        if self.is_running() {
            self.inner.task_ctl.signal(()).await?;
        }
        Ok(())
    }
}
