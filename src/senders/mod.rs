mod amex;
pub use amex::AmexSender;

mod api;
pub use api::APISender;

mod blob;
pub use blob::BlobSender;

mod sftp;
pub use sftp::SFTPSender;

use color_eyre::Result;

pub trait Sender {
    fn send(&self, transactions: String) -> Result<()>;
}
