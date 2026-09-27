use russh::client::{self, Handler};
use russh::keys::PublicKeyOrCertificate;
use tokio::sync::mpsc;

pub struct SshClientHandler {
    pub output_tx: mpsc::Sender<Vec<u8>>,
}

impl Handler for SshClientHandler {
    type Error = russh::Error;

    async fn check_server_key(
        &mut self,
        _server_public_key: &PublicKeyOrCertificate,
    ) -> Result<bool, Self::Error> {
        // Accept host key (in future: verify against known_hosts or user prompt)
        Ok(true)
    }

    async fn data(
        &mut self,
        _channel: russh::ChannelId,
        data: &[u8],
        _session: &mut client::Session,
    ) -> Result<(), Self::Error> {
        let _ = self.output_tx.send(data.to_vec()).await;
        Ok(())
    }
}
