// Port of https://www.rabbitmq.com/tutorials/tutorial-one-python.html. Run this
// in one shell, and run the hello_world_publish example in another.
use amiquip::{Connection, ConsumerMessage, ConsumerOptions, QueueDeclareOptions};
use atalanta::{configuration::load_settings, models::Transaction};
use color_eyre::Result;

fn main() -> Result<()> {
    let settings = load_settings()?;

    // Open connection.
    let mut connection = Connection::insecure_open(&settings.amqp_dsn)?;

    // Open a channel - None says let the library choose the channel ID.
    let channel = connection.open_channel(None)?;

    // Declare the "hello" queue.
    let queue = channel.queue_declare("new_transaction", QueueDeclareOptions::default())?;

    // Start a consumer.
    let consumer = queue.consume(ConsumerOptions::default())?;
    println!("Waiting for messages. Press Ctrl-C to exit.");

    for (i, message) in consumer.receiver().iter().enumerate() {
        match message {
            ConsumerMessage::Delivery(delivery) => {
                let body: Transaction = rmp_serde::from_slice(&delivery.body).unwrap();
                println!("(Received {},  {:?}", i, body);
                consumer.ack(delivery)?;
            }
            other => {
                println!("Consumer ended: {:?}", other);
                break;
            }
        }
    }

    connection.close()?;

    Ok(())
}
