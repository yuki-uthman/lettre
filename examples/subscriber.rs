#[tokio::main]
async fn main() {
    let client = reqwest::Client::new();

    let uuid = uuid::Uuid::new_v4();
    let body = format!("name=hello%20world&email={}%40gmail.com", uuid);
    let response = client
        .post(&format!("{}/subscriptions", "http://localhost:8000"))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.");

    println!("{:#?}", response);
}
