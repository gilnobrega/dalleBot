mod dalle;

use std::env;

use serde_json::json;
use serde_json::Value;
use serenity::async_trait;
use serenity::framework::standard::macros::{command, group};
use serenity::framework::standard::Args;
use serenity::framework::standard::{CommandResult, StandardFramework};
use serenity::model::channel::Message;
use serenity::prelude::*;

use crate::dalle::download_response_image;

#[group]
#[commands(ping)]
#[commands(text2img)]
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
async fn text2img(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if msg.author.id.0 == 835342160370728970 {
        let dalle_token = env::var("DALLE_TOKEN").expect("token");

        let response = &crate::dalle::text2img(args.message(), &dalle_token).await;

        let downloads = download_response_image(response).await;

        for download in downloads {
            let f = [(&download[..], "image.png")];

            msg.channel_id
                .send_message(&ctx.http, |m| {
                    // Reply to the given message
                    m.reference_message(msg);

                    // Ping the replied user
                    m.allowed_mentions(|am| {
                        am.replied_user(true);
                        am
                    });

                    // Attach image
                    m.files(f);

                    m
                })
                .await?;
        }
    } else {
        msg.reply(&ctx.http, "You do not have permission to use this command")
            .await?;
    }

    Ok({})
}
