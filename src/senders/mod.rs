mod amex;
pub use amex::AmexSender;

mod api;
pub use api::APISender;

mod blob;
pub use blob::BlobSender;

mod sftp;
pub use sftp::SFTPSender;

use color_eyre::Result;

use crate::models::SenderConfig;

pub trait Sender: TryFrom<SenderConfig> {
    fn send(&self, transactions: String) -> Result<()>;
}
