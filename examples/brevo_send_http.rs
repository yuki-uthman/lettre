use letter::domain::Person;
use letter::email::Brevo;

#[tokio::main]
async fn main() {
    let brevo = Brevo::with_secret(".secret");

    let time = chrono::Local::now().format("%Y-%m-%d %H:%M").to_string();
    let recipient = Person::parse("Yuki".to_string(), "yuki07yuki@gmail.com".to_string()).unwrap();
    let subject = format!("Hello, world! {}", time);
    let email = brevo
        .email_builder()
        .to(&recipient)
        .subject(&subject)
        .html_content("<h1>Hello, world!</h1>")
        .build();

    let res = brevo.send_email(&email).await.unwrap();

    println!("{:#?}", res);
}
