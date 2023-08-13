use std::env;

use serenity::async_trait;
use serenity::framework::standard::macros::group;
use serenity::framework::StandardFramework;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::model::prelude::{ReactionType, UserId};
use serenity::prelude::*;

struct BotState {
    pub counting_channel: u64,
    pub last_num: i128,
    pub last_counter: Option<UserId>,
    pub high_score: i128,
}
struct BotStateKey;
impl TypeMapKey for BotStateKey {
    type Value = Mutex<BotState>;
}
struct Handler;

#[group]
struct General;
#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        let data = ctx.data.read().await;
        let mut state = data.get::<BotStateKey>().unwrap().lock().await;
        if msg.channel_id == state.counting_channel {
            if let Ok(num) = msg.content.split(" ").collect::<Vec<_>>()[0].parse::<i128>() {
                if state.last_counter.is_some() && state.last_counter.unwrap() == msg.author.id {
                    let _ = msg
                        .reply(ctx.http, "You can't count twice dumbass. Next number is 1")
                        .await;
                    state.last_counter = None;
                    state.last_num = 0;
                    return;
                }
                if num == state.last_num + 1 {
                    if num > state.high_score {
                        let _ = msg
                            .react(ctx.http, ReactionType::Unicode("☑️".to_string()))
                            .await;
                        state.high_score = num
                    } else {
                        let _ = msg
                            .react(ctx.http, ReactionType::Unicode("✅".to_string()))
                            .await;
                    }

                    state.last_num = num;
                    state.last_counter = Some(msg.author.id);
                } else {
                    let _ = msg
                        .reply(
                            ctx.http,
                            "You fucked up, can't even count fuck you mf. Next number is 1",
                        )
                        .await;
                    state.last_num = 0;
                    state.last_counter = None;
                }
            }
        }
    }
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[tokio::main]
async fn main() {
    let framework = StandardFramework::new()
        .configure(|c| c.prefix("!c"))
        .group(&GENERAL_GROUP);

    let token = env::var("DISCORD_TOKEN").expect("token");
    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(&token, intents)
        .framework(framework)
        .event_handler(Handler)
        .await
        .expect("Err creating client");

    let bot_state = BotState {
        last_num: 0,
        last_counter: None,
        high_score: 0,
        counting_channel: env::var("COUNTING").expect("channel").parse().unwrap()
    };
    client
        .data
        .write()
        .await
        .insert::<BotStateKey>(Mutex::new(bot_state));

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
