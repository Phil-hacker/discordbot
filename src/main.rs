extern crate tokio;
mod battle;
mod battleresult;
mod choice;
mod game;
mod helper;
use crate::game::Game;
use dotenv::dotenv;
use rand::random;
use serenity::async_trait;
use serenity::builder::CreateComponents;
use serenity::model::prelude::ReactionType;
use serenity::utils::MessageBuilder;
use serenity::{
    client::EventHandler,
    model::application::{
        command::Command, component::ButtonStyle, interaction::Interaction,
        interaction::InteractionResponseType,
    },
    model::gateway::Ready,
    prelude::{Context, GatewayIntents},
    Client,
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};

struct Handler {
    games: RwLock<HashMap<String, Arc<Mutex<Game>>>>,
}

trait New {
    fn new() -> Self;
}

impl New for Handler {
    fn new() -> Self {
        Handler {
            games: RwLock::new(HashMap::new()),
        }
    }
}

impl Handler {
    fn delete_game(&self, id: &String) -> bool {
        if let Ok(games) = self.games.write().as_deref_mut() {
            return games.remove(id).is_some();
        }
        false
    }
    fn new_game(&self, rounds: u64) -> Option<String> {
        let id = random::<u128>().to_string();
        if let Ok(games) = self.games.write().as_deref_mut() {
            games.insert(
                id.clone(),
                Arc::new(Mutex::new(Game::new(id.clone(), rounds))),
            );
            return Some(id);
        }
        None
    }
    fn get_game(&self, id: &String) -> Option<Arc<Mutex<Game>>> {
        if let Ok(games) = self.games.read() {
            return Some(games.get(id)?.to_owned());
        }
        None
    }
}

fn generate_game_buttons<'a>(
    components: &'a mut CreateComponents,
    id: &String,
) -> &'a mut CreateComponents {
    components.create_action_row(|row| {
        row.create_button(|button| {
            button
                .label("Rock")
                .emoji(ReactionType::Unicode("🪨".to_string()))
                .style(ButtonStyle::Secondary)
                .custom_id(&format!("#r:{}", id))
        });
        row.create_button(|button| {
            button
                .label("Paper")
                .emoji(ReactionType::Unicode("📄".to_string()))
                .style(ButtonStyle::Secondary)
                .custom_id(&format!("#p:{}", id))
        });
        row.create_button(|button| {
            button
                .label("Scissors")
                .emoji(ReactionType::Unicode("✂️".to_string()))
                .style(ButtonStyle::Secondary)
                .custom_id(&format!("#s:{}", id))
        });
        row.create_button(|button| {
            button
                .label("Lizard")
                .emoji(ReactionType::Unicode("🦎".to_string()))
                .style(ButtonStyle::Secondary)
                .custom_id(&format!("#l:{}", id))
        });
        row.create_button(|button| {
            button
                .label("Spock")
                .emoji(ReactionType::Unicode("🖖".to_string()))
                .style(ButtonStyle::Secondary)
                .custom_id(&format!("#S:{}", id))
        })
    })
}

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        match interaction {
            Interaction::ApplicationCommand(command) => {
                let cmd = command.data.name.as_str();
                let mut options = command.data.options.clone();
                match cmd {
                    "rockpaperscissors" => {
                        let mut rounds = options
                            .drain(0..)
                            .filter_map(|option| {
                                if option.name == "rounds" {
                                    return option.value?.as_u64();
                                }
                                None
                            })
                            .sum();
                        if rounds < 1 {
                            rounds = 1;
                        }
                        println!("{}", &rounds);
                        if let Err(why) = command
                            .create_interaction_response(&ctx.http, |response| {
                                response
                                    .kind(InteractionResponseType::ChannelMessageWithSource)
                                    .interaction_response_data(|message| {
                                        match self.new_game(rounds) {
                                            Some(id) => {
                                                self.get_game(&id)
                                                    .unwrap()
                                                    .lock()
                                                    .unwrap()
                                                    .add_player(&command.user);
                                                message
                                                    .content(
                                                        self.get_game(&id)
                                                            .unwrap()
                                                            .lock()
                                                            .unwrap()
                                                            .generate_message(),
                                                    )
                                                    .components(|components| {
                                                        components.create_action_row(|row| {
                                                            row.create_button(|button| {
                                                                button.label("Join").custom_id(
                                                                    &format!("join:{}", id),
                                                                )
                                                            });
                                                            row.create_button(|button| {
                                                                button.label("Start").custom_id(
                                                                    &format!("start:{}", id),
                                                                )
                                                            })
                                                        })
                                                    });
                                                return message;
                                            }
                                            None => message.content("Ein Fehler ist aufgetreten"),
                                        }
                                    })
                            })
                            .await
                        {
                            println!("Cannot respond to slash command: {}", why);
                        }
                    },
                    "rules" => {
                        if let Err(why) = command.create_interaction_response(&ctx.http, |response| {
                            response.kind(InteractionResponseType::ChannelMessageWithSource).interaction_response_data(|message| {
                                message.embed(|embed| {
                                    embed.description(MessageBuilder::new()
                                    .push("Paper covers Rock\n")
                                    .push("Rock crushes Scissors\n")
                                    .push("Scissors decapitates Lizard\n")
                                    .push("Lizard poisons Spock\n")
                                    .push("Spock vaporizes Rock\n")
                                    .push("Rock crushes Lizard\n")
                                    .push("Lizard eats Paper\n")
                                    .push("Paper disproves Spock\n")
                                    .push("Spock smashes Scissors\n")
                                    .push("Scissors cuts Paper\n")
                                    .build())
                                }
                                )
                            })
                        }).await {
                            println!("Cannot respond to component {}", why);
                        };
                    },
                    _ => {}
                }
            }
            Interaction::MessageComponent(component) => {
                let user_id = &component.user;
                let id: String;
                let cmd: String;
                {
                    let cid = component.data.custom_id.clone();
                    let custom_id = match cid.split_once(':') {
                        Some(data) => data,
                        None => return,
                    };
                    id = custom_id.1.to_string();
                    cmd = custom_id.0.to_string();
                }
                if let Err(why) = component
                    .create_interaction_response(&ctx.http, |response| {
                        let game_arc = self.get_game(&id).unwrap();
                        let mut game = game_arc.lock().unwrap();
                        if cmd.eq(&"start".to_string()) {
                            if game.get_player_count() >= 2 {
                                game.start_round();
                                response
                                    .kind(InteractionResponseType::UpdateMessage)
                                    .interaction_response_data(|message| {
                                        message
                                            .content(game.generate_message())
                                            .components(|components| {
                                                generate_game_buttons(components, &id)
                                            })
                                    })
                            } else {
                                response.kind(InteractionResponseType::UpdateMessage)
                            }
                        } else if cmd.eq(&"join".to_string()) {
                            if game.add_player(user_id) {
                                response
                                    .kind(InteractionResponseType::UpdateMessage)
                                    .interaction_response_data(|message| {
                                        message.content(game.generate_message())
                                    })
                            } else {
                                response.kind(InteractionResponseType::UpdateMessage)
                            }
                        } else if cmd.chars().next().unwrap_or('+') == '#' {
                            game.choose(user_id, cmd.chars().nth(1).unwrap_or(' '));
                            response
                                .kind(InteractionResponseType::UpdateMessage)
                                .interaction_response_data(|message| {
                                    if game.did_all_choose() {
                                        let msg = game.battle();
                                        message.content(MessageBuilder::new().push(msg).build());
                                        if game.is_done() {
                                            message.set_components(CreateComponents(vec![]));
                                            drop(game);
                                            self.delete_game(&id);
                                        }
                                        message
                                    } else {
                                        message.content(game.generate_message())
                                    }
                                })
                        } else {
                            response.kind(InteractionResponseType::UpdateMessage)
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
        let _commands = Command::create_global_application_command(&ctx.http, |command| {
            command
                .name("rockpaperscissors")
                .add_option(
                    serenity::builder::CreateApplicationCommandOption(HashMap::from([]))
                        .name("rounds")
                        .description("how many rounds are played")
                        .kind(serenity::model::prelude::command::CommandOptionType::Integer)
                        .min_int_value(1)
                        .max_int_value(15)
                        .clone(),
                )
                .description("Start a new Game")
        })
        .await;
        let _commands = Command::create_global_application_command(&ctx.http, |command| {
            command
                .name("rules")
                .description("Get the rules of the game.")
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
