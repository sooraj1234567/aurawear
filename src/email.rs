use lettre::{
    message::{header::ContentType, Mailbox, Message},
    transport::smtp::{
        authentication::Credentials,
        AsyncSmtpTransport,
    },
    AsyncTransport,
    Tokio1Executor,
};

pub async fn send_reply_email(
    to_email: &str,
    name: &str,
    ticket_id: &str,
    original_msg: &str,
    admin_reply: &str,
) -> Result<(), String> {
    /*
     * Gmail SMTP configuration.
     *
     * These values MUST come from environment variables.
     * Do not put your Gmail password directly in the source code.
     */

    let smtp_username = std::env::var("SMTP_USER")
        .map_err(|_| {
            "SMTP_USER environment variable is not set".to_string()
        })?;

    let smtp_password = std::env::var("SMTP_PASS")
        .map_err(|_| {
            "SMTP_PASS environment variable is not set".to_string()
        })?;

    let smtp_from = std::env::var("SMTP_FROM")
        .unwrap_or_else(|_| smtp_username.clone());

    let from_name = "AURAWEAR Support";

    let subject =
        format!("AURAWEAR Reply - Ticket {}", ticket_id);

    /*
     * Escape basic HTML characters so user/admin text
     * cannot accidentally break the email HTML.
     */
    let escape_html = |value: &str| -> String {
        value
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&#39;")
    };

    let safe_name = escape_html(name);
    let safe_original_msg = escape_html(original_msg);
    let safe_admin_reply = escape_html(admin_reply);
    let safe_ticket_id = escape_html(ticket_id);

    let html_body = format!(
        r#"
<!DOCTYPE html>
<html>

<head>
    <meta charset="UTF-8">
    <meta name="viewport"
          content="width=device-width, initial-scale=1.0">

    <title>AURAWEAR Reply</title>
</head>

<body style="
    margin:0;
    padding:0;
    background:#f7f4ef;
    font-family:Arial,Helvetica,sans-serif;
    color:#1a1611;
">

<div style="
    max-width:620px;
    margin:40px auto;
    background:#ffffff;
    border:1px solid #e5e0d8;
">

    <div style="
        padding:28px 32px;
        border-bottom:1px solid #e5e0d8;
        text-align:center;
    ">

        <div style="
            font-size:22px;
            letter-spacing:5px;
            font-weight:bold;
        ">
            AURAWEAR
        </div>

        <div style="
            margin-top:8px;
            font-size:11px;
            letter-spacing:2px;
            color:#777;
        ">
            CLOTHING WITH AN AURA
        </div>

    </div>


    <div style="
        padding:36px 32px;
    ">

        <p style="
            margin:0 0 18px;
            font-size:16px;
        ">
            Hi <strong>{}</strong>,
        </p>

        <p style="
            color:#555;
            line-height:1.7;
        ">
            Thank you for contacting AURAWEAR.
            Our support team has replied to your message.
        </p>


        <div style="
            margin:26px 0;
            padding:18px;
            background:#f8f6f2;
            border-left:3px solid #1a1611;
        ">

            <div style="
                font-size:11px;
                letter-spacing:1.5px;
                color:#777;
                margin-bottom:8px;
            ">
                TICKET ID
            </div>

            <strong>{}</strong>

        </div>


        <div style="
            margin:26px 0;
        ">

            <div style="
                font-size:11px;
                letter-spacing:1.5px;
                color:#777;
                margin-bottom:10px;
            ">
                YOUR MESSAGE
            </div>

            <div style="
                padding:16px;
                background:#fafafa;
                border:1px solid #eeeeee;
                line-height:1.7;
                white-space:pre-wrap;
            ">
                {}
            </div>

        </div>


        <div style="
            margin:30px 0;
        ">

            <div style="
                font-size:11px;
                letter-spacing:1.5px;
                color:#777;
                margin-bottom:10px;
            ">
                ADMIN REPLY
            </div>

            <div style="
                padding:20px;
                background:#1a1611;
                color:#ffffff;
                line-height:1.8;
                white-space:pre-wrap;
            ">
                {}
            </div>

        </div>


        <p style="
            color:#555;
            line-height:1.7;
        ">
            If you need any further assistance,
            simply reply to this email or contact
            AURAWEAR again through our website.
        </p>


        <p style="
            margin-top:30px;
            line-height:1.7;
        ">
            Thanks,<br>
            <strong>AURAWEAR Support</strong>
        </p>

    </div>


    <div style="
        padding:20px 32px;
        border-top:1px solid #e5e0d8;
        text-align:center;
        font-size:11px;
        color:#888;
    ">
        AURAWEAR — Clothing With An Aura
    </div>

</div>

</body>
</html>
"#,
        safe_name,
        safe_ticket_id,
        safe_original_msg,
        safe_admin_reply
    );

    /*
     * Create the sender mailbox.
     *
     * SMTP_FROM can be used to control the From address.
     */
    let from_mailbox: Mailbox = format!(
        "{} <{}>",
        from_name,
        smtp_from
    )
    .parse()
    .map_err(|e| {
        format!(
            "Invalid SMTP sender email address: {}",
            e
        )
    })?;

    /*
     * Create recipient mailbox.
     */
    let to_mailbox: Mailbox = to_email
        .parse()
        .map_err(|e| {
            format!(
                "Invalid recipient email address '{}': {}",
                to_email,
                e
            )
        })?;

    /*
     * Build the email.
     */
    let email = Message::builder()
        .from(from_mailbox)
        .to(to_mailbox)
        .subject(subject)
        .header(ContentType::TEXT_HTML)
        .body(html_body)
        .map_err(|e| {
            format!(
                "Failed to build email: {}",
                e
            )
        })?;

    /*
     * Gmail SMTP credentials.
     *
     * SMTP_PASS must be a Gmail App Password,
     * NOT your normal Gmail account password.
     */
    let credentials = Credentials::new(
        smtp_username.clone(),
        smtp_password,
    );

    /*
     * Gmail SMTP server.
     *
     * Port 465 uses implicit TLS.
     */
    let mailer =
        AsyncSmtpTransport::<Tokio1Executor>::relay(
            "smtp.gmail.com"
        )
        .map_err(|e| {
            format!(
                "Failed to configure Gmail SMTP: {}",
                e
            )
        })?
        .credentials(credentials)
        .port(465)
        .build();

    /*
     * Send the email.
     */
    match mailer.send(email).await {

        Ok(_) => {
            println!(
                "Gmail SMTP email successfully sent to {}",
                to_email
            );

            Ok(())
        }

        Err(e) => {
            eprintln!(
                "Gmail SMTP email failed for {}: {}",
                to_email,
                e
            );

            Err(
                format!(
                    "Gmail SMTP error: {}",
                    e
                )
            )
        }
    }
}