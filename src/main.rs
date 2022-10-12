#[macro_use]
extern crate tokio;

use dotenv::dotenv;
use rand::random;
use serenity::async_trait;
use serenity::model::prelude::{component, UserId};
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
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::sync::Mutex;
enum Choices {
    Rock,
    Paper,
    Scissors,
    Lizard,
    Spock,
}

#[derive(PartialEq, Eq)]
struct Game {
    id: String,
    players: HashSet<u64>,
}

impl Game {
    fn new(id: String) -> Self {
        Game {
            id: id,
            players: HashSet::new(),
        }
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

impl Handler {
    fn start_game(&self, id: String) {

    }
    fn add_player(&self, id: String, user_id: u64) -> bool {
        match self.games.lock().as_deref_mut() {
            Ok(games) => {
                if let Some(game) = games.get_mut(&id) {
                    if !game.players.contains(&user_id) {
                        game.players.insert(user_id);
                        return true;
                    }
                }
                false
            }
            Err(_) => false,
        }
    }
    fn new_game(&self) -> Option<String> {
        let id = random::<u128>().to_string();
        match self.games.lock().as_deref_mut() {
            Ok(games) => {
                games.insert(id.clone(), Game::new(id.clone()));
                Some(id)
            }
            Err(_) => None,
        }
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
                                .interaction_response_data(|message| match self.new_game() {
                                    Some(id) => {
                                        message.content("React to join\nPlayers:\n").components(|components| {
                                            components.create_action_row(|row| {
                                                row.create_button(|button| {
                                                    button.label("Join").custom_id(&id)
                                                });
                                                row.create_button(|button| {
                                                    button.label("Start").custom_id(&format!("start{}",id))
                                                })
                                            })
                                        })
                                    }
                                    None => message.content("Ein Fehler ist aufgetreten"),
                                })
                        })
                        .await
                    {
                        println!("Cannot respond to slash command: {}", why);
                    }
                }
            }
            Interaction::MessageComponent(component) => {
                let user_id=*component.user.id.as_u64();
                let id=component.data.custom_id.clone();
                let content = component.message.content.clone();
                if let Err(why) = component
                    .create_interaction_response(&ctx.http, |response| {
                        if id.contains("start") {
                            self.start_game(id);
                            response
                            .kind(InteractionResponseType::UpdateMessage)
                            .interaction_response_data(|message| {
                                message.content({
                                    MessageBuilder::new()
                                        .push("Choose your weapon")
                                        .build()
                                })
                            })
                        }else {
                        if self.add_player(id,user_id) {
                            response
                                .kind(InteractionResponseType::UpdateMessage)
                                .interaction_response_data(|message| {
                                    message.content({
                                        MessageBuilder::new()
                                            .push(content)
                                            .push("\n")
                                            .mention(&component.user)
                                            .build()
                                    })
                                })
                        } else {
                            response.kind(InteractionResponseType::UpdateMessage)
                        }
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
