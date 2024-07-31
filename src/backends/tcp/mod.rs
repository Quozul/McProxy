use tokio::task::JoinHandle;
use tracing::error;

mod tcp_proxy;

pub(crate) fn start_tcp_proxy(listen: String, redirect: String) -> JoinHandle<()> {
    tokio::spawn(async move {
        let proxy = tcp_proxy::start_tcp_proxy(listen, redirect).await;
        if let Err(err) = proxy {
            error!("error with TCP proxy; error={err}");
        }
    })
}
