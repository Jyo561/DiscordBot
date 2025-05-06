use std::env;
use std::fs::{OpenOptions};
use std::io::Write;
use std::error::Error;

use chrono::Local;
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use dotenv::dotenv;
use serde::{Serialize,Deserialize};
//use reqwest::Bytes;
use bytes::Bytes;

struct Handler;

#[derive(Debug, Serialize, Deserialize)]
struct Expense {
    amount: f64,
    place: String,
    purpose: String,
    spend_date: String,
    //type: String,
}



#[async_trait]
impl EventHandler for Handler {
    // Set a handler for the `message` event. This is called whenever a new message is received.
    //
    // Event handlers are dispatched through a threadpool, and so multiple events can be
    // dispatched simultaneously.
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content.starts_with("$add ") {
            // Sending a message can fail, due to a network error, an authentication error, or lack
            // of permissions to post in the channel, so log to stdout when some error happens,
            // with a description of it.
            let parts: Vec<&str> = msg.content.split(' ').map(|s| s.trim()).collect();
            if parts.len() != 4 {
                let _ = msg.channel_id.say(&ctx.http, "❗ Please use the format: `$add amount place purpose`").await;
                return;
            }

            let amount: i32 = parts[1].parse().unwrap_or(0);
            let place = parts[2];
            let purpose = parts[3];
            let spend_date = Local::now().format("%Y-%m-%d").to_string();

            let file_name = format!("expense/{}.txt", spend_date);
            let entry = format!("{},{},{},{}\n", amount, place, purpose, spend_date);

            let mut file = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(file_name)
                    .expect("Failed to open log file");

            file.write_all(entry.as_bytes()).expect("Failed to write to file");


            /*let response = if parts.len() > 1 {
                format!("👋 Hello {}!", parts[1])
            } else {
                "👋 Hello!".to_string()
            };*/
            let response = format!("✅ Expense saved:\nAmount: {}\nPlace: {}\nPurpose: {}\nDate: {}", amount, place, purpose, spend_date);
            
            if let Err(why) = msg.channel_id.say(&ctx.http, response).await {
                println!("Error sending message: {why:?}");
            }
        }
        else if msg.content.starts_with("$addimage"){
            if let Some(attachment) = msg.attachments.first() {
                if let Some(content_type) = &attachment.content_type {
                    if content_type.starts_with("image/") {
                        // 1. Download image
                        match reqwest::get(&attachment.url).await {
                            Ok(resp) => match resp.bytes().await {
                                Ok(image_bytes) => {
                            // 2. Send to Gemini API
                                    match call_gemini_api(&image_bytes).await {
                                        Ok(result_text) => {
                                            let result_text_cloned = result_text.clone(); // clone to use inside spawn_blocking

                                            let write_result = tokio::task::spawn_blocking(move || -> Result<(), std::io::Error> {
                                                let today = chrono::Local::now().format("%Y-%m-%d").to_string();
                                                let file_path = format!("expense/{}.txt", today);

                                                std::fs::create_dir_all("expense").expect("Error");

                                                let mut file = std::fs::OpenOptions::new()
                                                        .create(true)
                                                        .append(true)
                                                        .open(&file_path).expect("Error");

                                                for line in result_text_cloned.lines() {
                                                    if !line.trim().is_empty() {
                                                        writeln!(file, "{}", line.trim()).expect("Error");
                                                    }
                                                }

                                                Ok(())
                                            }).await;

                                            match write_result {
                                                Ok(Ok(())) => {
                                                    msg.channel_id.say(&ctx.http, "✅ Expenses saved!").await.ok();
                                                }
                                                Ok(Err(e)) => {
                                                    msg.channel_id.say(&ctx.http, format!("❌ File write error: {}", e)).await.ok();
                                                }
                                                Err(e) => {
                                                    msg.channel_id.say(&ctx.http, format!("❌ Task join error: {}", e)).await.ok();
                                                }

                                            }

                                            //msg.channel_id.say(&ctx.http, format!("🧾 Gemini Response:\n{}", result_text)).await.ok();
                                            msg.channel_id.say(&ctx.http, format!("🧾 Gemini Response:\n{}", result_text)).await.ok();
                                        },
                                        Err(e) => {
                                            msg.channel_id.say(&ctx.http, format!("❌ Gemini API error: {}", e)).await.ok();
                                        }
                                    }
                                },
                                Err(_) => {
                                    msg.channel_id.say(&ctx.http, "❌ Failed to read image bytes.").await.ok();
                                }
                            },
                            Err(_) => {
                                msg.channel_id.say(&ctx.http, "❌ Failed to download image.").await.ok();
                            }
                        }
                    } else {
                        msg.channel_id.say(&ctx.http, "⚠️ Only image files are accepted.").await.ok();
                    }
                }
            } else {
                msg.channel_id.say(&ctx.http, "📎 Please upload an image.").await.ok();
            }
        } 
        else if msg.content.starts_with("$total"){
            let mut total_amount = 0;
            let expense_dir = std::path::Path::new("expense");

            if expense_dir.exists() {
                if let Ok(entries) = std::fs::read_dir(expense_dir) {
                    for entry_result in entries {
                        if let Ok(entry) = entry_result {
                            let path = entry.path();
                            if path.extension().map_or(false, |ext| ext == "txt") {
                                if let Ok(file) = std::fs::File::open(&path) {
                                    let reader = std::io::BufReader::new(file);
                                    use std::io::BufRead; // bring trait into scope

                                    for line_result in reader.lines() {
                                        if let Ok(line) = line_result {
                                            let parts: Vec<&str> = line.split(',').collect();
                                            if let Some(amount_str) = parts.get(0) {
                                                if let Ok(amount) = amount_str.trim().parse::<i32>() {
                                                    total_amount += amount;
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            let response = format!("💰 Net Total Spent: ₹{}", total_amount);
            msg.channel_id.say(&ctx.http, response).await.ok();
        }
        else if msg.content.starts_with("$"){}
    }

    // Set a handler to be called on the `ready` event. This is called when a shard is booted, and
    // a READY payload is sent by Discord. This payload contains data like the current user's guild
    // Ids, current user data, private channels, and more.
    //
    // In this case, just print what the current user's username is.
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

async fn call_gemini_api(image_bytes: &Bytes) -> Result<String, Box<dyn Error + Send + Sync>> {
    let api_key = std::env::var("GEMINI_API_KEY")?;
    let image_b64 = base64::encode(image_bytes);

    let body = serde_json::json!({
        "contents": [{
            "parts": [
            {
                "text": "Extract the expenses from this bill or receipt. For each item, return it as: amount,place,purpose on a new line. If multiple items exist, list each on a separate line."
            },
            {
                "inline_data": {
                    "mime_type": "image/png",
                    "data": image_b64
                }
            }]
        }]
    });

    let client = reqwest::Client::new();
    let res = client
        .post("https://generativelanguage.googleapis.com/v1beta/models/gemini-2.0-flash:generateContent")
        .query(&[("key", &api_key)])
        .json(&body)
        .send()
        .await?;

    let json: serde_json::Value = res.json().await?;
    println!("{:?}",json);
    let text = json["candidates"][0]["content"]["parts"][0]["text"]
        .as_str()
        .unwrap_or("⚠️ No text extracted")
        .to_string();

    Ok(text)
}


#[tokio::main]
async fn main() {
    // Configure the client with your Discord bot token in the environment.
    
    dotenv::dotenv().ok();

    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MESSAGES
    | GatewayIntents::DIRECT_MESSAGES
    | GatewayIntents::MESSAGE_CONTENT;

    // Create a new instance of the Client, logging in as a bot. This will automatically prepend
    // your bot token with "Bot ", which is a requirement by Discord for bot users.
    let mut client =
        Client::builder(&token, intents).event_handler(Handler).await.expect("Err creating client");

    // Finally, start a single shard, and start listening to events.
    //
    // Shards will automatically attempt to reconnect, and will perform exponential backoff until
    // it reconnects.
    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}
