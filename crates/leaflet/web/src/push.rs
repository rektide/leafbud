use crate::{Error, LeafletClient, transport};
use leafbud_protocol::EncodedPush;

/// HTTP/RPC acceptance, explicitly NOT a mutation acknowledgment or achievement.
#[derive(Debug)]
pub struct PushAccepted;

impl LeafletClient {
    /// Send the immutable prepared bytes. Retry this SAME value in-process;
    /// never encode new Yrs state under its client/mutation IDs.
    pub async fn send_push(&self, request: &EncodedPush) -> Result<PushAccepted, Error> {
        let response = self
            .http
            .post(self.base.join("api/rpc/push").expect("static path"))
            .header("Content-Type", "application/json")
            .body(request.bytes().to_owned())
            .send()
            .await
            .map_err(transport)?;
        let raw = self.read_json(response, "POST push").await?;
        if !raw.is_object() || raw.as_object().is_some_and(|o| o.keys().any(|k| k != "result")) || raw.get("result").is_some_and(|v| !v.is_null()) {
            return Err(Error::Native("push RPC error or unsupported result; observe writer acknowledgments"));
        }
        Ok(PushAccepted)
    }
}
