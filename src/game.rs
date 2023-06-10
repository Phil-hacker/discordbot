use crate::battle::Battle;
use crate::battleresult::BattleResult;
use crate::choice::Choice;
use crate::helper::choice_to_emoji;
use crate::helper::get_choice_from_char;
use itertools::Itertools;
use serenity::builder::CreateEmbed;
use serenity::model::user::User;
use serenity::utils::MessageBuilder;
use std::collections::{HashMap, HashSet};

#[derive(PartialEq, Eq)]
pub struct Game {
    started: bool,
    round: u64,
    rounds: u64,
    id: String,
    players: HashSet<User>,
    choices: HashMap<User, Choice>,
    points: HashMap<User, u64>,
}

impl Game {
    pub fn new(id: String, rounds: u64) -> Self {
        Game {
            started: false,
            round: 0,
            rounds,
            id,
            players: HashSet::new(),
            choices: HashMap::new(),
            points: HashMap::new(),
        }
    }
    pub fn get_player_count(&self) -> usize {
        self.players.len()
    }
    pub fn add_player(&mut self, user: &User) -> bool {
        if !self.players.contains(user) && !self.started {
            self.players.insert(user.clone());
            self.points.insert(user.clone(), 0);
            return true;
        }
        false
    }
    pub fn get_finished_players(&self) -> usize {
        return self.choices.keys().count();
    }
    pub fn did_all_choose(&self) -> bool {
        self.get_player_count() == self.get_finished_players()
    }
    pub fn did_all_choose_same(&self) -> Option<Choice> {
        if self.choices.values().dedup().count() == 1 {
            if let Some(v) = self.choices.values().next() {
                return Some(*v);
            }
        }
        None
    }
    pub fn choose(&mut self, user: &User, c: char) {
        if self.players.contains(user) {
            let choice = get_choice_from_char(c);
            if let Some(choice) = choice {
                self.choices.insert(user.clone(), choice);
            }
        }
    }
    pub fn get_all_interactions(&self) -> Vec<BattleResult> {
        self.choices
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
            .collect()
    }
    pub fn battle(&mut self) -> () {
        self.choices.drain();
        self.round += 1;
    }
    pub fn is_done(&self) -> bool {
        self.round >= self.rounds
    }
    fn add_point(&mut self, user: &User) {
        self.points
            .insert(user.clone(), *self.points.get(user).unwrap_or(&0) + 1);
    }
    fn generate_point_list(&self) -> String {
        self.points
            .clone()
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
            .collect::<String>()
    }
    pub fn generate_embed<'a>(&mut self, embed: &'a mut CreateEmbed) -> &'a mut CreateEmbed {
        if !self.started {
            embed
                .title("Rock Paper Scissors Lizard Spock")
                .description(format!("Rounds:{}", self.rounds));
            let mut msg = MessageBuilder::new();
            msg.push(format!("Players:\n"));
            self.players.iter().for_each(|user| {
                msg.mention(user).push("\n");
            });
            return embed.field("Players", msg.build(), false);
        }
        if !self.did_all_choose() {
            let mut msg = MessageBuilder::new();
            if self.get_rounds() > 1 {
                msg.push("round ")
                    .push(self.get_round())
                    .push("/")
                    .push(self.get_rounds())
                    .push("\n");
            }
            embed.description(msg
                .push("Choose your weapon\n")
                .push(self.get_finished_players())
                .push("/")
                .push(self.get_player_count())
                .push(" players chose")
                .build());
        } else {
            let msg = {
                if let Some(choice) = self.did_all_choose_same() {
                    MessageBuilder::new()
                        .push("All players chose ")
                        .push(choice_to_emoji(choice))
                        .push("\nNo one wins")
                        .build()
                } else {
                    MessageBuilder::new()
                        .push(
                            self.get_all_interactions()
                                .into_iter()
                                .map(|battle| {
                                    self.add_point(&battle.winner);
                                    battle.to_message()
                                })
                                .collect::<String>(),
                        )
                        .build()
                }
            };
            embed.field("Choices", msg, false);
        }
        embed.field("Points", self.generate_point_list(), false);
        embed.title("Rock Paper Scissors Lizard Spock")
    }
    pub fn start_round(&mut self) {
        self.started = true;
    }
    pub fn get_round(&self) -> u64 {
        self.round + 1
    }
    pub fn get_rounds(&self) -> u64 {
        self.rounds
    }
}
