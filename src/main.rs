#[macro_use]
extern crate tokio;

use dotenv::dotenv;
use rand::random;
use serenity::async_trait;
use serenity::model::prelude::UserId;
use serenity::utils::MessageBuilder;
use serenity::{
    builder::CreateActionRow,
    client::EventHandler,
    model::application::{
        command::Command, interaction::Interaction, interaction::InteractionResponseType,
    },
    model::gateway::Ready,
    prelude::{Context, GatewayIntents},
    Client,
};
use std::collections::HashMap;
use std::sync::Mutex;
enum Choices {
    Rock,
    Paper,
    Scissors,
    Lizard,
    Spock,
}

#[derive(PartialEq, Eq, Hash)]
struct Game {
    players: Vec<UserId>,
}

impl Game {
    fn new() -> Self {
        Game { players: vec![] }
    }
}

struct Handler {
    games: Mutex<HashMap<String, Game>>,
}

trait New {
    fn new() -> Self;
}

impl New for Handler {
    fn new() -> Self {
        return Handler {
            games: Mutex::new(HashMap::new()),
        };
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        match interaction {
            Interaction::ApplicationCommand(command) => {
                let cmd = command.data.name.as_str();
                if cmd == "rockpaperscissors" {
                    if let Err(why) = command
                        .create_interaction_response(&ctx.http, |response| {
                            response
                                .kind(InteractionResponseType::ChannelMessageWithSource)
                                .interaction_response_data(|message| {
                                    let id = random::<u128>().to_string();
                                    match self.games.lock().as_deref_mut() {
                                        Ok(games) => {
                                            games.insert("test".to_string(), Game::new());
                                            message.content("React to join").components(
                                                |components| {
                                                    components.create_action_row(|row| {
                                                        row.create_button(|button| {
                                                            button.label("Join").custom_id(&id)
                                                        })
                                                    })
                                                },
                                            )
                                        }
                                        Err(_) => message.content("Ein Fehler ist aufgetreten"),
                                    }
                                })
                        })
                        .await
                    {
                        println!("Cannot respond to slash command: {}", why);
                    }
                }
            }
            Interaction::MessageComponent(component) => {
                if let Err(why) = component
                    .create_interaction_response(&ctx.http, |response| {
                        match self.games.lock().as_deref_mut() {
                            Ok(games) => response
                                .kind(InteractionResponseType::ChannelMessageWithSource)
                                .interaction_response_data(|message| {
                                    message.content({
                                        MessageBuilder::new()
                                            .mention(&component.user)
                                            .push(" joined the Game")
                                            .build()
                                    })
                                }),
                            Err(_) => response,
                        }
                    })
                    .await
                {
                    println!("Cannot respond to component {}", why);
                };
            }
            _ => {}
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
        let commands = Command::create_global_application_command(&ctx.http, |command| {
            command
                .name("rockpaperscissors")
                .description("Start a new Game")
        })
        .await;
    }
}
#[tokio::main]
async fn main() {
    dotenv().unwrap();
    let token = std::env::var("DISCORD_TOKEN").unwrap();
    let mut client = Client::builder(
        token,
        GatewayIntents::GUILD_MESSAGE_REACTIONS.union(GatewayIntents::GUILD_MESSAGES),
    )
    .event_handler(Handler::new())
    .await
    .unwrap();
    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
    println!("Hello, world!");
}
