use axum_seaorm_app::config::mail::EmailSender;
use dotenvy::dotenv; // Load .env file
use futures_util::StreamExt; // For consumer.next()
use lapin::{Connection, ConnectionProperties, message::Delivery, options::*, types::FieldTable};
use std::env;

#[derive(serde::Serialize, serde::Deserialize)]
struct EmailJob {
    to: String,
    subject: String,
    body: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load environment variables from .env
    dotenv().ok();

    // RabbitMQ configuration
    let rabbit_user = env::var("RABBITMQ_USER").unwrap_or_else(|_| "admin".to_string());
    let rabbit_pass = env::var("RABBITMQ_PASS").unwrap_or_else(|_| "secret123".to_string());
    let rabbit_port = env::var("RABBITMQ_PORT").unwrap_or_else(|_| "5672".to_string());
    let rabbit_url = format!(
        "amqp://{}:{}@rabbitmq:{}/%2f",
        rabbit_user, rabbit_pass, rabbit_port
    );

    // Establish connection & channel
    let conn = Connection::connect(&rabbit_url, ConnectionProperties::default()).await?;
    let channel = conn.create_channel().await?;

    // Only one message at a time
    channel.basic_qos(1, BasicQosOptions::default()).await?;

    // Declare durable email queue
    channel
        .queue_declare(
            "email_queue",
            QueueDeclareOptions {
                durable: true,
                ..Default::default()
            },
            FieldTable::default(),
        )
        .await?;

    // Start consuming
    let mut consumer = channel
        .basic_consume(
            "email_queue",
            "email_worker",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await?;

    println!("[*] Waiting for email jobs…");

    while let Some(result) = consumer.next().await {
        match result {
            Ok(delivery) => {
                if let Err(e) = handle_delivery(&channel, delivery).await {
                    eprintln!("Error handling delivery: {e}");
                }
            }
            Err(e) => eprintln!("Consumer error: {e}"),
        }
    }

    Ok(())
}

/// Handles one message from RabbitMQ
async fn handle_delivery(channel: &lapin::Channel, delivery: Delivery) -> anyhow::Result<()> {
    let job: EmailJob = serde_json::from_slice(&delivery.data)?;
    println!("Sending email to: {}", job.to);

    // Create EmailSender from environment
    let sender = EmailSender::new()?;

    if let Err(e) = sender.send_email(&job.to, &job.subject, &job.body).await {
        eprintln!("Failed to send email: {e}");
    } else {
        println!("Email sent successfully to {}", job.to);
    }

    // Ack message
    channel
        .basic_ack(delivery.delivery_tag, BasicAckOptions::default())
        .await?;

    Ok(())
}
