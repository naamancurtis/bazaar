use anyhow::Error;
use tracing::error;

pub fn generate_error_log(error: Error, message: Option<&str>) {
    let mut error_chain = error.chain().collect::<Vec<_>>();
    if let Some(root_cause) = error_chain.pop() {
        error!(?root_cause, ?error_chain, "{}", message.unwrap_or_default());
    }
}
