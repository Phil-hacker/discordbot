extern crate tokio;
mod choice;
use crate::choice::Choice;
mod battleresult;
use crate::battleresult::BattleResult;
mod helper;
use crate::helper::choice_to_emoji;
mod battle;
use crate::battle::Battle;
mod game;
use crate::game::Game;
use dotenv::dotenv;
use itertools::Itertools;
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
    fn generate_message(&self, id: &String) -> String {
        let mut points: HashMap<User, u16> = HashMap::new();
        if let Some(choice) = self.did_all_choose_same(id) {
            return MessageBuilder::new()
                .push("All players chose ")
                .push(choice_to_emoji(choice))
                .push("\nNo one wins")
                .build();
        } else if let Some(battles) = self.get_all_interactions(id) {
            return MessageBuilder::new()
                .push(
                    battles
                        .into_iter()
                        .map(|battle| {
                            points.insert(
                                battle.winner.clone(),
                                *points.get(&battle.winner).unwrap_or(&0) + 1,
                            );
                            points.insert(
                                battle.loser.clone(),
                                *points.get(&battle.loser).unwrap_or(&0),
                            );
                            battle.to_message()
                        })
                        .collect::<String>(),
                )
                .push(
                    points
                        .into_iter()
                        .sorted_by_key(|user| user.1)
                        .rev()
                        .into_iter()
                        .map(|user| {
                            MessageBuilder::new()
                                .mention(&user.0)
                                .push(format!(" {} points", user.1))
                                .push("\n")
                                .build()
                        })
                        .collect::<String>(),
                )
                .build();
        }
        "".to_string()
    }
    fn get_all_interactions(&self, id: &String) -> Option<Vec<BattleResult>> {
        if let Ok(games) = self.games.lock().as_deref_mut() {
            let battle_results: Vec<BattleResult> = games
                .get(id)?
                .choices
                .iter()
                .collect_vec()
                .into_iter()
                .combinations(2)
                .filter_map(|combination| {
                    Some(Battle::new_ref(
                        combination.get(0)?.0,
                        combination.get(0)?.1,
                        combination.get(1)?.0,
                        combination.get(1)?.1,
                    ))
                })
                .filter_map(|battle| battle.battle())
                .dedup()
                .collect();
            return Some(battle_results);
        }
        None
    }
    fn did_all_choose_same(&self, id: &String) -> Option<Choice> {
        if let Ok(games) = self.games.lock().as_deref() {
            if let Some(game) = games.get(id) {
                if game.choices.values().dedup().count() == 1 {
                    if let Some(v) = game.choices.values().next() {
                        return Some(*v);
                    }
                }
            }
        }
        None
    }
    fn choose(&self, id: &String, user: &User, c: char) {
        if let Ok(games) = self.games.lock().as_deref_mut() {
            if let Some(game) = games.get_mut(id) {
                if game.players.contains(user) {
                    let choice = get_choice_from_char(c);
                    if let Some(choice) = choice {
                        game.choices.insert(user.clone(), choice);
                    }
                }
            }
        }
    }

    fn get_finished_players(&self, id: &String) -> usize {
        if let Ok(games) = self.games.lock().as_deref_mut() {
            if let Some(game) = games.get(id) {
                return game.choices.keys().count();
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
                game.started = true;
                return true;
            }
        }
        false
    }
    fn did_all_choose(&self, id: &String) -> bool {
        if let Ok(games) = self.games.lock().as_deref() {
            if let Some(game) = games.get(id) {
                return game.players.len() == game.choices.keys().count();
            }
        }
        false
    }
    fn get_player_count(&self, id: &String) -> usize {
        if let Ok(games) = self.games.lock().as_deref() {
            if let Some(game) = games.get(id) {
                return game.players.len();
            }
        }
        0
    }
    fn add_player(&self, id: String, user: &User) -> bool {
        if let Ok(games) = self.games.lock().as_deref_mut() {
            if let Some(game) = games.get_mut(&id) {
                if !game.players.contains(user) && !game.started {
                    game.players.insert(user.clone());
                    return true;
                }
            }
        }
        false
    }
    fn new_game(&self) -> Option<String> {
        let id = random::<u128>().to_string();
        if let Ok(games) = self.games.lock().as_deref_mut() {
            games.insert(id.clone(), Game::new(id.clone()));
            return Some(id);
        }
        None
    }
}

fn get_choice_from_char(c: char) -> Option<Choice> {
    match c {
        'r' => Some(Choice::Rock),
        'p' => Some(Choice::Paper),
        's' => Some(Choice::Scissors),
        'l' => Some(Choice::Lizard),
        'S' => Some(Choice::Spock),
        _ => None,
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
                                    Some(id) => message
                                        .content("React to join\nPlayers:\n")
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
                                        }),
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
                            if self.add_player(id, user_id) {
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
                                        message
                                            .content(
                                                MessageBuilder::new()
                                                    .push(self.generate_message(&id))
                                                    .build(),
                                            )
                                            .components(|components| components);
                                        self.delete_game(&id);
                                        message
                                    } else {
                                        message.content(format!(
                                            "Choose your weapon\n{}/{} players chose",
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
