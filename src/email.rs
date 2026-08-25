use reqwest::Client;
use serde_json::json;
use std::env;

pub async fn send_reply_email(
    to_email: &str,
    name: &str,
    ticket_id: &str,
    original_message: &str,
    admin_reply: &str,
) -> Result<(), String> {
    let api_key = env::var("RESEND_API_KEY").map_err(|_| "RESEND_API_KEY environment variable is not set")?;
    
    let client = Client::new();
    let url = "https://api.resend.com/emails";

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
            <p>You can view your message history and order tracking anytime in your <a href="https://aurawear-o0eq.onrender.com/profile.html">AuraWear Profile</a>.</p>
            <p style="font-size:11px; opacity:0.5; margin-top:30px;">EST. 1972 • REIMAGINED TODAY</p>
        </div>
        "#,
        name, ticket_id, original_message, admin_reply
    );

    let payload = json!({
        "from": "AuraWear Support <onboarding@resend.dev>",
        "to": [to_email],
        "subject": format!("Update on your AuraWear Ticket #{}", ticket_id),
        "html": html_content
    });

    let res = client.post(url)
        .bearer_auth(api_key)
        .json(&payload)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if res.status().is_success() {
        Ok(())
    } else {
        let err_text = res.text().await.unwrap_or_default();
        Err(format!("Resend API error: {}", err_text))
    }
}