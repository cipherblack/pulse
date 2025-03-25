use lettre::message::header::ContentType;
use lettre::{Message, SmtpTransport, Transport};
use lettre::transport::smtp::authentication::Credentials;
use colored::Colorize;

pub async fn send(to: &str, cpu_usage: f32) {
    let email = Message::builder()
        .from("myemail@gmail.com".parse().unwrap()) // ایمیل خودت
        .to(to.parse().unwrap())
        .subject("SysPulse Alert: High CPU Usage")
        .header(ContentType::TEXT_PLAIN)
        .body(format!("Warning: CPU usage is at {:.2}%!", cpu_usage))
        .unwrap();

    let creds = Credentials::new("myemail@gmail.com".to_string(), "your-16-char-app-password".to_string()); // رمز برنامه

    let mailer = SmtpTransport::relay("smtp.gmail.com")
        .unwrap()
        .credentials(creds)
        .build();

    match mailer.send(&email) {
        Ok(_) => println!("{}", "Email alert sent!".green()),
        Err(e) => eprintln!("{}", format!("Failed to send email: {:?}", e).red()),
    }
}