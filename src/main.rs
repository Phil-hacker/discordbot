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
use serenity::model::prelude::ReactionType;
use serenity::model::user::User;
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
use std::sync::Mutex;

struct Handler {
    games: Mutex<HashMap<String, Game>>,
}

trait New {
    fn new() -> Self;
}

impl New for Handler {
    fn new() -> Self {
        Handler {
            games: Mutex::new(HashMap::new()),
        }
    }
}

impl Handler {
    fn delete_game(&self, id: &String) -> bool {
        if let Ok(games) = self.games.lock().as_deref_mut() {
            return games.remove(id).is_some();
        }
        false
    }
    fn generate_message(&self, id: &String) -> (String, bool) {
        if let Ok(games) = self.games.lock().as_deref_mut() {
            if let Some(game) = games.get_mut(id) {
                let mut msg = game.battle();
                if game.is_done() {
                    msg.push_str("\n Ende der Runde");
                }
                return (msg, game.is_done());
            }
        }
        ("".to_string(), false)
    }
    fn choose(&self, id: &String, user: &User, c: char) {
        if let Ok(games) = self.games.lock().as_deref_mut() {
            if let Some(game) = games.get_mut(id) {
                game.choose(user, c);
            }
        }
    }
    fn get_finished_players(&self, id: &String) -> usize {
        if let Ok(games) = self.games.lock().as_deref_mut() {
            if let Some(game) = games.get(id) {
                return game.get_finished_players();
            }
        }
        0
    }
    fn start_game(&self, id: &String) -> bool {
        if self.get_player_count(id) < 2 {
            return false;
        }
        if let Ok(games) = self.games.lock().as_deref_mut() {
            if let Some(game) = games.get_mut(id) {
                game.start_round();
                return true;
            }
        }
        false
    }
    fn get_round(&self, id: &String) -> u64 {
        if let Ok(games) = self.games.lock().as_deref() {
            if let Some(game) = games.get(id) {
                return game.get_round();
            }
        }
        1
    }
    fn get_rounds(&self, id: &String) -> u64 {
        if let Ok(games) = self.games.lock().as_deref() {
            if let Some(game) = games.get(id) {
                return game.get_rounds();
            }
        }
        1
    }
    fn did_all_choose(&self, id: &String) -> bool {
        if let Ok(games) = self.games.lock().as_deref() {
            if let Some(game) = games.get(id) {
                return game.did_all_choose();
            }
        }
        false
    }
    fn get_player_count(&self, id: &String) -> usize {
        if let Ok(games) = self.games.lock().as_deref() {
            if let Some(game) = games.get(id) {
                return game.get_player_count();
            }
        }
        0
    }
    fn add_player(&self, id: &String, user: &User) -> bool {
        if let Ok(games) = self.games.lock().as_deref_mut() {
            if let Some(game) = games.get_mut(id) {
                return game.add_player(user);
            }
        }
        false
    }
    fn new_game(&self, rounds: u64) -> Option<String> {
        let id = random::<u128>().to_string();
        if let Ok(games) = self.games.lock().as_deref_mut() {
            games.insert(id.clone(), Game::new(id.clone(), rounds));
            return Some(id);
        }
        None
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        match interaction {
            Interaction::ApplicationCommand(command) => {
                let cmd = command.data.name.as_str();
                let mut options = command.data.options.clone();
                if cmd == "rockpaperscissors" {
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
                                .interaction_response_data(|message| match self.new_game(rounds) {
                                    Some(id) => {self.add_player(&id, &command.user);message
                                        .content(format!(
                                            "Rounds:{}\nPlayers:\n",
                                            self.get_rounds(&id)
                                        ))
                                        .components(|components| {
                                            components.create_action_row(|row| {
                                                row.create_button(|button| {
                                                    button
                                                        .label("Join")
                                                        .custom_id(&format!("join:{}", id))
                                                });
                                                row.create_button(|button| {
                                                    button
                                                        .label("Start")
                                                        .custom_id(&format!("start:{}", id))
                                                })
                                            })
                                        })},
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
                let content = component.message.content.clone();
                if let Err(why) = component
                    .create_interaction_response(&ctx.http, |response| {
                        if cmd.eq(&"start".to_string()) {
                            if self.start_game(&id) {
                                response
                                    .kind(InteractionResponseType::UpdateMessage)
                                    .interaction_response_data(|message| {
                                        message
                                            .content(format!(
                                                "Choose your weapon\n{}/{} players chose",
                                                self.get_finished_players(&id),
                                                self.get_player_count(&id)
                                            ))
                                            .components(|components| {
                                                components.create_action_row(|row| {
                                                    row.create_button(|button| {
                                                        button
                                                            .label("Rock")
                                                            .emoji(ReactionType::Unicode(
                                                                "🪨".to_string(),
                                                            ))
                                                            .style(ButtonStyle::Secondary)
                                                            .custom_id(&format!("#r:{}", id))
                                                    });
                                                    row.create_button(|button| {
                                                        button
                                                            .label("Paper")
                                                            .emoji(ReactionType::Unicode(
                                                                "📄".to_string(),
                                                            ))
                                                            .style(ButtonStyle::Secondary)
                                                            .custom_id(&format!("#p:{}", id))
                                                    });
                                                    row.create_button(|button| {
                                                        button
                                                            .label("Scissors")
                                                            .emoji(ReactionType::Unicode(
                                                                "✂️".to_string(),
                                                            ))
                                                            .style(ButtonStyle::Secondary)
                                                            .custom_id(&format!("#s:{}", id))
                                                    });
                                                    row.create_button(|button| {
                                                        button
                                                            .label("Lizard")
                                                            .emoji(ReactionType::Unicode(
                                                                "🦎".to_string(),
                                                            ))
                                                            .style(ButtonStyle::Secondary)
                                                            .custom_id(&format!("#l:{}", id))
                                                    });
                                                    row.create_button(|button| {
                                                        button
                                                            .label("Spock")
                                                            .emoji(ReactionType::Unicode(
                                                                "🖖".to_string(),
                                                            ))
                                                            .style(ButtonStyle::Secondary)
                                                            .custom_id(&format!("#S:{}", id))
                                                    })
                                                })
                                            })
                                    })
                            } else {
                                response.kind(InteractionResponseType::UpdateMessage)
                            }
                        } else if cmd.eq(&"join".to_string()) {
                            if self.add_player(&id, user_id) {
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
                        } else if cmd.chars().next().unwrap_or('+') == '#' {
                            self.choose(&id, user_id, cmd.chars().nth(1).unwrap_or(' '));
                            response
                                .kind(InteractionResponseType::UpdateMessage)
                                .interaction_response_data(|message| {
                                    if self.did_all_choose(&id) {
                                        let msg = self.generate_message(&id);
                                        message.content(MessageBuilder::new().push(msg.0).build());
                                        if msg.1 {
                                            message.components(|components| components);
                                            self.delete_game(&id);
                                        }

                                        message
                                    } else {
                                        message.content(format!(
                                            "Round {}/{}\nChoose your weapon\n{}/{} players chose",
                                            self.get_round(&id),
                                            self.get_rounds(&id),
                                            self.get_finished_players(&id),
                                            self.get_player_count(&id)
                                        ))
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
