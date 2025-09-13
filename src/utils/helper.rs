use std::env;

#[allow(dead_code)]
pub fn get_rabbitmq_url() -> String {
    let rabbit_user = env::var("RABBITMQ_USER").unwrap_or_else(|_| "admin".to_string());
    let rabbit_pass = env::var("RABBITMQ_PASS").unwrap_or_else(|_| "secret123".to_string());
    let rabbit_port = env::var("RABBITMQ_PORT").unwrap_or_else(|_| "5672".to_string());

    // vhost অন্তর্ভুক্ত করে প্রোগ্রাম্যাটিকভাবে URL তৈরি করা হয়
    format!(
        "amqp://{}:{}@rabbitmq:{}/%2f",
        rabbit_user, rabbit_pass, rabbit_port
    )
}
