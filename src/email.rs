use serde_json::json;

pub async fn send_reply_email(
    to_email: &str,
    name: &str,
    ticket_id: &str,
    original_msg: &str,
    admin_reply: &str,
) -> Result<(), String> {
    let api_key = std::env::var("RESEND_API_KEY")
        .map_err(|_| "RESEND_API_KEY environment variable is not set".to_string())?;

    let from_email = "AuraWear Support <onboarding@resend.dev>";
    let subject = format!("AURAWEAR Reply - Ticket {}", ticket_id);
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

    let client = reqwest::Client::new();
    let res = client
        .post("https://api.resend.com/emails")
        .bearer_auth(api_key)
        .json(&json!({
            "from": from_email,
            "to": [to_email],
            "subject": subject,
            "html": html_body
        }))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if res.status().is_success() {
        println!("Resend email successfully sent to {}", to_email);
        Ok(())
    } else {
        let err_text = res.text().await.unwrap_or_default();
        eprintln!("Failed to send email via Resend: {}", err_text);
        Err(err_text)
    }
}