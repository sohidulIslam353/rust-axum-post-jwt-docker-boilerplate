use crate::errors::MyError;
use lapin::{BasicProperties, Connection, ConnectionProperties, options::*, types::FieldTable};
use serde::Serialize;
use std::env;

#[allow(dead_code)]
#[derive(Serialize)]
pub struct EmailJob {
    pub to: String,
    pub subject: String,
    pub body: String,
}

#[allow(dead_code)]
pub async fn publish_to_queue(task: &EmailJob, queue: &str) -> Result<(), MyError> {
    // Retrieve individual RabbitMQ connection parts from environment variables
    let rabbit_user = env::var("RABBITMQ_USER").unwrap_or_else(|_| "admin".to_string());
    let rabbit_pass = env::var("RABBITMQ_PASS").unwrap_or_else(|_| "secret123".to_string());
    let rabbit_port = env::var("RABBITMQ_PORT").unwrap_or_else(|_| "5672".to_string());

    // Construct the URL programmatically to ensure vhost is included
    let rabbitmq_url = format!(
        "amqp://{}:{}@rabbitmq:{}/%2f",
        rabbit_user, rabbit_pass, rabbit_port
    );

    // Connect to RabbitMQ using the URL from the environment
    let connection = Connection::connect(&rabbitmq_url, ConnectionProperties::default()).await?;
    let channel = connection.create_channel().await?;

    // Declare the queue (ensure it exists and is durable)
    channel
        .queue_declare(
            queue,
            QueueDeclareOptions {
                durable: true,
                ..Default::default()
            },
            FieldTable::default(),
        )
        .await?;

    // Serialize the task into bytes
    let payload = serde_json::to_vec(task).map_err(MyError::SerdeJsonError)?;

    // Publish the task to the queue
    channel
        .basic_publish(
            "",    // Default exchange
            queue, // Queue name
            BasicPublishOptions::default(),
            &payload,                   // Pass the reference to the payload
            BasicProperties::default(), // No extra properties, but you can set properties here
        )
        .await?;

    Ok(())
}
