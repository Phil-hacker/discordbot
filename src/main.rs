extern crate tokio;

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
use std::collections::{HashMap, HashSet};
use std::sync::Mutex;
#[derive(PartialEq, Eq, Clone, Copy)]
enum Choice {
    Rock,
    Paper,
    Scissors,
    Lizard,
    Spock,
}

#[derive(PartialEq)]
struct BattleResult {
    winner: User,
    winner_choice: Choice,
    loser: User,
    loser_choice: Choice,
    verb: String,
}
impl BattleResult {
    fn new(
        winner: User,
        winner_choice: Choice,
        loser: User,
        loser_choice: Choice,
        verb: &str,
    ) -> Self {
        BattleResult {
            winner,
            winner_choice,
            loser,
            loser_choice,
            verb: verb.to_string(),
        }
    }
}

struct Battle {
    player1: User,
    choice1: Choice,
    player2: User,
    choice2: Choice,
}

impl Battle {
    fn new(player1: User, choice1: Choice, player2: User, choice2: Choice) -> Self {
        Battle {
            player1,
            choice1,
            player2,
            choice2,
        }
    }
    fn new_ref(player1: &User, choice1: &Choice, player2: &User, choice2: &Choice) -> Self {
        Self::new(player1.clone(), *choice1, player2.clone(), *choice2)
    }
    fn battle(&self) -> Option<BattleResult> {
        match self.choice1 {
            Choice::Rock => match self.choice2 {
                Choice::Rock => None,
                Choice::Paper => Some(self.win2("covers")),
                Choice::Scissors => Some(self.win1("crushes")),
                Choice::Lizard => Some(self.win1("crushes")),
                Choice::Spock => Some(self.win2("vaporizes")),
            },
            Choice::Paper => match self.choice2 {
                Choice::Rock => Some(self.win1("covers")),
                Choice::Paper => None,
                Choice::Scissors => Some(self.win2("cuts")),
                Choice::Lizard => Some(self.win2("eats")),
                Choice::Spock => Some(self.win1("disproves")),
            },
            Choice::Scissors => match self.choice2 {
                Choice::Rock => Some(self.win2("crushes")),
                Choice::Paper => Some(self.win1("cuts")),
                Choice::Scissors => None,
                Choice::Lizard => Some(self.win1("decapitates")),
                Choice::Spock => Some(self.win2("smashes")),
            },
            Choice::Lizard => match self.choice2 {
                Choice::Rock => Some(self.win2("crushes")),
                Choice::Paper => Some(self.win1("eats")),
                Choice::Scissors => Some(self.win2("decapitates")),
                Choice::Lizard => None,
                Choice::Spock => Some(self.win1("poisons")),
            },
            Choice::Spock => match self.choice2 {
                Choice::Rock => Some(self.win1("vaporizes")),
                Choice::Paper => Some(self.win2("disproves")),
                Choice::Scissors => Some(self.win1("smashes")),
                Choice::Lizard => Some(self.win2("poisons")),
                Choice::Spock => None,
            },
        }
    }
    fn win1(&self, verb: &str) -> BattleResult {
        BattleResult::new(
            self.player1.clone(),
            self.choice1,
            self.player2.clone(),
            self.choice2,
            verb,
        )
    }
    fn win2(&self, verb: &str) -> BattleResult {
        BattleResult::new(
            self.player2.clone(),
            self.choice2,
            self.player1.clone(),
            self.choice1,
            verb,
        )
    }
}

#[derive(PartialEq, Eq)]
struct Game {
    started: bool,
    id: String,
    players: HashSet<User>,
    choices: HashMap<User, Choice>,
}

impl Game {
    fn new(id: String) -> Self {
        Game {
            started: false,
            id,
            players: HashSet::new(),
            choices: HashMap::new(),
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
        Handler {
            games: Mutex::new(HashMap::new()),
        }
    }
}

impl Handler {
    fn generate_message(&self, id: &String) -> String {
        let mut points: HashMap<User, u16> = HashMap::new();
        if let Some(choice) = self.did_all_choose_same(id) {
            return MessageBuilder::new()
            .push("All players chose ")
            .push(choice_to_emoji(choice))
            .push("\nNo one wins")
            .build()
        } else {
            if let Some(battles) = self.get_all_interactions(id) {
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
                                MessageBuilder::new()
                                    .mention(&battle.winner)
                                    .push(choice_to_emoji(battle.winner_choice))
                                    .push(" ")
                                    .push(battle.verb)
                                    .push(" ")
                                    .push(choice_to_emoji(battle.loser_choice))
                                    .mention(&battle.loser)
                                    .push("\n")
                                    .build()
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
        }
        "".to_string()
    }
    fn get_all_interactions(&self, id: &String) -> Option<Vec<BattleResult>> {
        if let Ok(games) = self.games.lock().as_deref_mut() {
            let battle_results: Vec<BattleResult> = games
                .get(id)?
                .choices
                .iter()
                .cartesian_product(games.get(id)?.choices.iter())
                .filter_map(|combination| {
                    if combination.0 .0 == combination.1 .0 {
                        None
                    } else {
                        Some(Battle::new_ref(
                            combination.0 .0,
                            combination.0 .1,
                            combination.1 .0,
                            combination.1 .1,
                        ))
                    }
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

fn choice_to_emoji(c: Choice) -> ReactionType {
    match c {
        Choice::Rock => ReactionType::Unicode("🪨".to_string()),
        Choice::Paper => ReactionType::Unicode("📄".to_string()),
        Choice::Scissors => ReactionType::Unicode("✂️".to_string()),
        Choice::Lizard => ReactionType::Unicode("🦎".to_string()),
        Choice::Spock => ReactionType::Unicode("🖖".to_string()),
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
                                            .components(|components| components)
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
