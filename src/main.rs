mod dalle_api;

use dalle_api::get_credits;
use serde_json::Value;
use serenity::async_trait;
use serenity::framework::standard::macros::{command, group};
use serenity::framework::standard::Args;
use serenity::framework::standard::{CommandResult, StandardFramework};
use serenity::model::channel::Message;
use serenity::model::prelude::AttachmentType;
use serenity::prelude::*;
use std::env;

use crate::dalle_api::get_response_image_urls;

#[group]
#[commands(text2img, ping, credits)]
struct General;

struct Handler;

#[async_trait]
impl EventHandler for Handler {}

#[tokio::main]
async fn main() {
    let framework = StandardFramework::new()
        .configure(|c| c.prefix("~")) // set the bot's prefix to "~"
        .group(&GENERAL_GROUP);

    // Login with a bot token from the environment
    let discord_token = env::var("DISCORD_TOKEN").expect("token");

    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;
    let mut client = Client::builder(discord_token, intents)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Error creating client");

    // start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }
}

#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    msg.reply(ctx, "Pong!").await?;

    Ok(())
}

#[command]
async fn text2img(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let dalle_token = env::var("DALLE_TOKEN").expect("token");
    let thinking_reaction = '🤔';
    msg.react(&ctx.http, thinking_reaction).await.unwrap();

    match &crate::dalle_api::text2img(args.message(), &dalle_token).await {
        Ok(response) => {
            msg.delete_reaction_emoji(&ctx.http, thinking_reaction)
                .await
                .unwrap();

            download_and_send_images(response, ctx, msg).await;

            let fut1 = msg.react(&ctx.http, '🟢');
            let fut2 = credits(&ctx, &msg, args);
            (fut1.await.unwrap(), fut2.await.unwrap());
        }
        Err(_) => {
            let fut1 = msg.delete_reaction_emoji(&ctx.http, thinking_reaction);
            let fut2 = msg.react(&ctx.http, '🔴');
            (fut1.await.unwrap(), fut2.await.unwrap());
        }
    };

    Ok({})
}

async fn download_and_send_images(response: &Value, ctx: &Context, msg: &Message) {
    let urls = get_response_image_urls(response).await;

    let emojis = ['🌑', '🌘', '🌗', '🌖', '🌕'];

    msg.react(&ctx.http, emojis[0]).await.unwrap();

    let mut files = Vec::new();

    for (i, url) in urls.iter().enumerate() {
        let download = reqwest::get(url).await.unwrap().bytes().await.unwrap();
        msg.delete_reaction_emoji(&ctx.http, emojis[i])
            .await
            .unwrap();
        msg.react(&ctx.http, emojis[i + 1]).await.unwrap();

        let f = download;
        files.push(f);
    }

    msg.channel_id
        .send_message(&ctx.http, |m| {
            // Reply to the given message
            m.reference_message(msg);

            // Ping the replied user
            m.allowed_mentions(|am| {
                am.replied_user(true);
                am
            });

            for (i, file) in files.iter().enumerate() {
                m.add_file(AttachmentType::Bytes { data: std::borrow::Cow::Borrowed(file), filename: format!("{}.webp", i) });
            }

            m
        })
        .await
        .unwrap();

    msg.delete_reaction_emoji(&ctx.http, emojis[4])
        .await
        .unwrap();
}

#[command]
async fn credits(ctx: &Context, msg: &Message) -> CommandResult {
    let dalle_token = env::var("DALLE_TOKEN").expect("token");

    let mut output = "Unable to get balance".to_string();

    match get_credits(&dalle_token).await {
        Ok(val) => match val {
            Some(val) => {
                let price = (18 as f64/ 115 as f64) * val as f64;
                output = format!("{0} credits left (approx ${1:.2})", val, price);
            }
            None => {}
        },
        Err(_) => {}
    };

    msg.reply(&ctx.http, output).await.unwrap();

    Ok(())
}
