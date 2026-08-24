use lettre::{Message, SmtpTransport, Transport};
use lettre::transport::smtp::authentication::Credentials;

pub async fn send_reply_email(to_email: &str, name: &str, ticket_id: &str, original_msg: &str, admin_reply: &str) -> Result<(), String> {
    let from_email = std::env::var("SMTP_FROM").unwrap_or("ammavishnu9605@gmail.com".to_string());
    let smtp_user = std::env::var("SMTP_USER").or_else(|_| std::env::var("SMTP_USERNAME")).unwrap_or(from_email.clone());
    let smtp_pass = std::env::var("SMTP_PASS").or_else(|_| std::env::var("SMTP_PASSWORD")).unwrap_or_default();
    let smtp_host = std::env::var("SMTP_HOST").unwrap_or("smtp.gmail.com".to_string());

    let from_addr = from_email.parse().map_err(|e: lettre::address::AddressError| e.to_string())?;
    let to_addr = to_email.parse().map_err(|e: lettre::address::AddressError| e.to_string())?;

    let email = Message::builder()
        .from(lettre::message::Mailbox::new(None, from_addr))
        .to(lettre::message::Mailbox::new(Some(name.to_string()), to_addr))
        .subject(format!("AURAWEAR Reply - Ticket {}", ticket_id))
        .body(format!(
            "Hi {},\n\nYou contacted AURAWEAR regarding: {}\n\nTicket ID: {}\nYour message: {}\n\nAdmin Reply:\n{}\n\nThanks,\nAURAWEAR - Clothing With An Aura",
            name, original_msg, ticket_id, original_msg, admin_reply
        ))
        .map_err(|e| e.to_string())?;

    let creds = Credentials::new(smtp_user, smtp_pass);
    
    // Explicitly configure TLS relay for port 587 or STARTTLS to prevent cloud blocks on Render
    let mailer = SmtpTransport::starttls_relay(&smtp_host)
        .map_err(|e| e.to_string())?
        .credentials(creds)
        .build();

    tokio::task::spawn_blocking(move || mailer.send(&email))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())?;

    Ok(())
}