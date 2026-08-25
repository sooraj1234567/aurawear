use reqwest::Client;
use serde_json::json;
use std::env;

pub async fn send_admin_notification(
    user_name: &str,
    user_email: &str,
    message: &str,
    contact_id: &str,
    image_url: &str,
) -> Result<(), String> {
    let api_key = env::var("BREVO_API_KEY")
        .map_err(|_| "BREVO_API_KEY environment variable is not set")?;
    
    let sender_email = env::var("SENDER_EMAIL")
        .unwrap_or_else(|_| "ammavishnu9605@gmail.com".to_string());

    let client = Client::new();
    let url = "https://api.v3.brevo.com/v3/smtp/email"; // or your endpoint url

    // If the user uploaded an image, create an HTML preview snippet for it
    let image_section = if !image_url.is_empty() {
        // Ensure the URL is absolute so Gmail can fetch and display it
        let full_img_url = if image_url.starts_with("http") {
            image_url.to_string()
        } else {
            format!("http://localhost:3000{}", image_url)
        };

        format!(
            r#"<p><b>Attached Image:</b><br/><a href="{}" target="_blank"><img src="{}" alt="User Attachment" style="max-width:300px; border-radius:4px; margin-top:8px; border:1px solid #ccc;"/></a></p>"#,
            full_img_url, full_img_url
        )
    } else {
        "".to_string()
    };

    let html_content = format!(
        r#"
        <div style="font-family:sans-serif; padding:20px; background:#FBF8F3; color:#1A1611;">
            <h2>AURAWEAR — New Contact Message</h2>
            <p>You have received a new message from a customer:</p>
            <div style="background:#fff; padding:15px; border-left:3px solid #C9A86A; margin:15px 0;">
                <p><b>Ticket ID:</b> #{}</p>
                <p><b>Name:</b> {}</p>
                <p><b>Email:</b> {}</p>
                <p><b>Message:</b><br/>{}</p>
                {}
            </div>
            <p style="font-size:11px; opacity:0.5; margin-top:30px;">Log in to your admin panel to reply directly.</p>
        </div>
        "#,
        contact_id, user_name, user_email, message, image_section
    );

    let payload = json!({
        "sender": {
            "name": "AuraWear Notifications",
            "email": sender_email
        },
        "to": [
            {
                "email": "ammavishnu9605@gmail.com",
                "name": "Admin Vishnu"
            }
        ],
        "subject": format!("New Contact Message from {} (#{})", user_name, contact_id),
        "htmlContent": html_content
    });

    let res = client.post("https://api.brevo.com/v3/smtp/email")
        .header("accept", "application/json")
        .header("api-key", api_key)
        .header("content-type", "application/json")
        .json(&payload)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if res.status().is_success() {
        Ok(())
    } else {
        let err_text = res.text().await.unwrap_or_default();
        Err(format!("Brevo API error (Admin Notification): {}", err_text))
    }
}

/// 2. Sends the admin reply to the user, and a copy to the admin brand email for records
pub async fn send_reply_email(
    to_email: &str,
    name: &str,
    ticket_id: &str,
    original_message: &str,
    admin_reply: &str,
) -> Result<(), String> {
    let api_key = env::var("BREVO_API_KEY")
        .map_err(|_| "BREVO_API_KEY environment variable is not set")?;
    
    let sender_email = env::var("SENDER_EMAIL")
        .unwrap_or_else(|_| "ammavishnu9605@gmail.com".to_string());

    let client = Client::new();
    let url = "https://api.brevo.com/v3/smtp/email";

    // Content for the User
    let user_html = format!(
        r#"
        <div style="font-family:sans-serif; padding:20px; background:#FBF8F3; color:#1A1611;">
            <h2>AURAWEAR — Official Support Reply</h2>
            <p>Hello <b>{}</b>,</p>
            <p>An administrator has replied to your contact ticket <b>#{}</b>:</p>
            <div style="background:#fff; padding:15px; border-left:3px solid #C9A86A; margin:15px 0;">
                <p style="margin:0 0 10px 0; opacity:0.7; font-size:12px;"><b>Your Message:</b> {}</p>
                <p style="margin:0; font-size:14px;"><b>Admin Reply:</b> {}</p>
            </div>
            <p>You can view your message history anytime in your <a href="https://aurawear-o0eq.onrender.com/profile.html">AuraWear Profile</a>.</p>
        </div>
        "#,
        name, ticket_id, original_message, admin_reply
    );

    let user_payload = json!({
        "sender": { "name": "AuraWear Support", "email": sender_email },
        "to": [{ "email": to_email, "name": name }],
        "subject": format!("Update on your AuraWear Ticket #{}", ticket_id),
        "htmlContent": user_html
    });

    // Send to User
    let user_res = client.post(url)
        .header("accept", "application/json")
        .header("api-key", &api_key)
        .header("content-type", "application/json")
        .json(&user_payload)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !user_res.status().is_success() {
        let err_text = user_res.text().await.unwrap_or_default();
        return Err(format!("Brevo API error (User Reply): {}", err_text));
    }

    // Content Copy for Admin Records / Sent Folder view
    let admin_copy_html = format!(
        r#"
        <div style="font-family:sans-serif; padding:20px;">
            <h2>[Sent Reply Record] Ticket #{}</h2>
            <p><b>Recipient:</b> {} ({})</p>
            <p><b>Original Message:</b> {}</p>
            <hr/>
            <p><b>Admin Reply Sent:</b> {}</p>
        </div>
        "#,
        ticket_id, name, to_email, original_message, admin_reply
    );

    let admin_copy_payload = json!({
        "sender": { "name": "AuraWear System", "email": sender_email },
        "to": [{ "email": "ammavishnu9605@gmail.com", "name": "Admin Record" }],
        "subject": format!("[Record] Reply sent for Ticket #{}", ticket_id),
        "htmlContent": admin_copy_html
    });

    // Send copy to Admin inbox/record
    let _ = client.post(url)
        .header("accept", "application/json")
        .header("api-key", api_key)
        .header("content-type", "application/json")
        .json(&admin_copy_payload)
        .send()
        .await;

    Ok(())
}