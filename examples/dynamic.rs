use std::{thread, time::Duration};

use color_eyre::Result;

/// A trait for sending messages somewhere.
trait Sender {
    fn send(&self, msg: Vec<&str>);
}

/// A struct that can send messages via SFTP.
struct SFTPSender {
    host: String,
    port: u16,
}

impl Sender for SFTPSender {
    fn send(&self, msg: Vec<&str>) {
        println!("SFTP: {msg:?} to {}:{}", self.host, self.port);
    }
}

/// A struct that can send messages via API.
struct APISender {
    url: String,
}

impl Sender for APISender {
    fn send(&self, msg: Vec<&str>) {
        println!("API: {} to {}", msg[0], self.url);
    }
}

/// A struct that can send messages to a blob storage.
struct BlobSender {
    container: String,
}

impl Sender for BlobSender {
    fn send(&self, msg: Vec<&str>) {
        println!("BLOB: {msg:?} to {}", self.container);
    }
}

trait Consumer {
    fn consume<F>(&self, f: F)
    where
        F: Fn(Vec<&str>);
}

// A consumer that reads messages off a queue and sends them immediately.
struct InstantConsumer;

impl Consumer for InstantConsumer {
    fn consume<F>(&self, f: F)
    where
        F: Fn(Vec<&str>),
    {
        let message = "Hello, world!"; // read off queue
        f(vec![message]);
    }
}

// A consumer that reads messages off a queue and sends them after a delay.
struct SleepyConsumer {
    delay: Duration,
}

impl Consumer for SleepyConsumer {
    fn consume<F>(&self, f: F)
    where
        F: Fn(Vec<&str>),
    {
        let message = "Hello, world!"; // read off queue
        thread::sleep(self.delay);
        f(vec![message]);
    }
}

/// A generic function that can send messages via any Sender.
fn send_message<C: Consumer, S: Sender>(consumer: &C, sender: &S) {
    consumer.consume(|msg| sender.send(msg));
}

fn main() -> Result<()> {
    color_eyre::install()?;

    // Get provider slug as first argument.
    let provider = std::env::args()
        .nth(1)
        .expect("Pass a provider slug as first argument");

    // Providers:
    // wasabi: SFTP
    // mastercard: Blob
    // visa: API
    match provider.as_str() {
        "wasabi" => {
            // TODO: use ScheduleConsumer { schedule: "*/15 * * * * *" }
            let consumer = InstantConsumer;
            let sender = SFTPSender {
                host: "sftp://wasabi.com".to_owned(),
                port: 22,
            };

            send_message(&consumer, &sender);
        }
        "mastercard" => {
            let consumer = SleepyConsumer {
                delay: Duration::from_secs(3),
            };
            let sender = BlobSender {
                container: "harmonia-transactions".to_owned(),
            };

            send_message(&consumer, &sender);
        }
        "visa" => {
            let consumer = InstantConsumer;
            let sender = APISender {
                url: "http://zephyrus.local/api".to_owned(),
            };

            send_message(&consumer, &sender);
        }
        _ => panic!("Unknown provider"),
    }

    Ok(())
}
