use std::path::Path;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixListener;
use crate::server::dispatcher::McpDispatcher;
use crate::server::protocol::JsonRpcRequest;

pub async fn run_socket_transport(
    socket_path: &Path,
    dispatcher: Arc<McpDispatcher>,
) -> Result<(), Box<dyn std::error::Error>> {
    if socket_path.exists() {
        let _ = std::fs::remove_file(socket_path);
    }

    let listener = UnixListener::bind(socket_path)?;

    // Set permissions to 0600 (owner read/write only)
    #[cfg(unix)]
    {
        use std::ffi::CString;
        if let Ok(c_path) = CString::new(socket_path.to_string_lossy().as_bytes()) {
            unsafe {
                libc::chmod(c_path.as_ptr(), 0o600);
            }
        }
    }

    loop {
        let (stream, _) = listener.accept().await?;
        let disp = Arc::clone(&dispatcher);

        tokio::spawn(async move {
            let (reader_half, mut writer_half) = stream.into_split();
            let mut lines = BufReader::new(reader_half).lines();

            while let Ok(Some(line)) = lines.next_line().await {
                if line.trim().is_empty() {
                    continue;
                }
                if let Ok(req) = serde_json::from_str::<JsonRpcRequest>(&line) {
                    let res = disp.dispatch(req).await;
                    if let Ok(mut out) = serde_json::to_string(&res) {
                        out.push('\n');
                        let _ = writer_half.write_all(out.as_bytes()).await;
                        let _ = writer_half.flush().await;
                    }
                }
            }
        });
    }
}
