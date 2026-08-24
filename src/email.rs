use lettre::{Message, SmtpTransport, Transport};
use lettre::transport::smtp::authentication::Credentials;
use lettre::message::header::ContentType;

pub async fn send_reply_email(to_email: &str, name: &str, ticket_id: &str, original_msg: &str, admin_reply: &str) -> Result<(), String> {
    let from_email = std::env::var("SMTP_FROM").unwrap_or("ammavishnu9605@gmail.com".to_string());
    let smtp_user = std::env::var("SMTP_USER").or_else(|_| std::env::var("SMTP_USERNAME")).unwrap_or(from_email.clone());
    let smtp_pass = std::env::var("SMTP_PASS").or_else(|_| std::env::var("SMTP_PASSWORD")).unwrap_or_default();
    let smtp_host = std::env::var("SMTP_HOST").unwrap_or("smtp.gmail.com".to_string());

    let from_addr = from_email.parse().map_err(|e: lettre::address::AddressError| e.to_string())?;
    let to_addr = to_email.parse().map_err(|e: lettre::address::AddressError| e.to_string())?;

    let html_body = format!(
        "<p>Hi <strong>{}</strong>,</p>\
         <p>You contacted AURAWEAR regarding: <em>{}</em></p>\
         <p><strong>Ticket ID:</strong> {}</p>\
         <p><strong>Your message:</strong> {}</p>\
         <hr/>\
         <p><strong>Admin Reply:</strong><br/>{}</p>\
         <br/>\
         <p>Thanks,<br/><strong>AURAWEAR - Clothing With An Aura</strong></p>",
        name, original_msg, ticket_id, original_msg, admin_reply
    );

    let email = Message::builder()
        .from(lettre::message::Mailbox::new(Some("AuraWear Support".to_string()), from_addr))
        .to(lettre::message::Mailbox::new(Some(name.to_string()), to_addr))
        .subject(format!("AURAWEAR Reply - Ticket {}", ticket_id))
        .header(ContentType::TEXT_HTML)
        .body(html_body)
        .map_err(|e| e.to_string())?;

    let creds = Credentials::new(smtp_user, smtp_pass);
    
    // Using starttls_relay ensures secure port 587 transmission required by cloud servers
    let mailer = SmtpTransport::starttls_relay(&smtp_host)
        .map_err(|e| e.to_string())?
        .credentials(creds)
        .build();

    let res = tokio::task::spawn_blocking(move || mailer.send(&email))
        .await
        .map_err(|e| e.to_string())?;

    match res {
        Ok(_) => {
            println!("Email successfully sent to {} via {}", to_email, smtp_host);
            Ok(())
        }
        Err(e) => {
            let err_msg = format!("SMTP Send Error: {:?}", e);
            eprintln!("{}", err_msg);
            Err(err_msg)
        }
    }
}