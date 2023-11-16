#[tokio::main]
async fn main() {
    let client = reqwest::Client::new();

    let uuid = uuid::Uuid::new_v4();
    let empty_name = format!("name=&email={}%40gmail.com", uuid);
    let response = client
        .post(&format!("{}/subscriptions", "http://localhost:8000"))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(empty_name)
        .send()
        .await
        .expect("Failed to execute request.");
    println!("{:#?}", response);

    let invalid_char = format!("name=%7Bhello%7D&email={}%40gmail.com", uuid);
    let response = client
        .post(&format!("{}/subscriptions", "http://localhost:8000"))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(invalid_char)
        .send()
        .await
        .expect("Failed to execute request.");

    println!("{:#?}", response);
}
