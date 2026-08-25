use lettre::message::header::ContentType;
use lettre::{Message, SmtpTransport, Transport};
use lettre::transport::smtp::authentication::Credentials;
use std::env;

pub async fn send_reply_email(
    to_email: &str,
    name: &str,
    ticket_id: &str,
    original_message: &str,
    admin_reply: &str,
) -> Result<(), String> {
    // Load environment variables for Gmail SMTP
    let smtp_host = env::var("SMTP_HOST").unwrap_or_else(|_| "smtp.gmail.com".to_string());
    let smtp_user = env::var("SMTP_USER").map_err(|_| "ammavishnu9605@gmail.com")?;
    let smtp_pass = env::var("SMTP_PASS").map_err(|_| "oqvh zrud vfgb jees")?;
    let smtp_from = env::var("SMTP_FROM").unwrap_or_else(|_| smtp_user.clone());

    let html_content = format!(
        r#"
        <div style="font-family:sans-serif; padding:20px; background:#FBF8F3; color:#1A1611;">
            <h2>AURAWEAR — Archive Support Reply</h2>
            <p>Hello <b>{}</b>,</p>
            <p>An administrator has replied to your contact ticket <b>#{}</b>:</p>
            <div style="background:#fff; padding:15px; border-left:3px solid #C9A86A; margin:15px 0;">
                <p style="margin:0 0 10px 0; opacity:0.7; font-size:12px;"><b>Your Message:</b> {}</p>
                <p style="margin:0; font-size:14px;"><b>Admin Reply:</b> {}</p>
            </div>
            <p>You can view your message history and order tracking anytime in your <a href="http://localhost:3000/profile.html">AuraWear Profile</a>.</p>
            <p style="font-size:11px; opacity:0.5; margin-top:30px;">EST. 1972 • REIMAGINED TODAY</p>
        </div>
        "#,
        name, ticket_id, original_message, admin_reply
    );

    let email = Message::builder()
        .from(smtp_from.parse().map_err(|e: lettre::address::AddressError| e.to_string())?)
        .to(to_email.parse().map_err(|e: lettre::address::AddressError| e.to_string())?)
        .subject(format!("Update on your AuraWear Ticket #{}", ticket_id))
        .header(ContentType::TEXT_HTML)
        .body(html_content)
        .map_err(|e| e.to_string())?;

    let creds = Credentials::new(smtp_user, smtp_pass);

    // Open a remote connection to Gmail SMTP server using STARTTLS
    let mailer = SmtpTransport::starttls_relay(&smtp_host)
        .map_err(|e| e.to_string())?
        .credentials(creds)
        .build();

    // Send the email synchronously in a blocking task to fit async boundaries
    let email_clone = email;
    tokio::task::spawn_blocking(move || {
        mailer.send(&email_clone)
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())?;

    Ok(())
}