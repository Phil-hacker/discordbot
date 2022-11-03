use crate::battle::Battle;
use crate::battleresult::BattleResult;
use crate::choice::Choice;
use crate::helper::choice_to_emoji;
use crate::helper::get_choice_from_char;
use itertools::Itertools;
use serenity::model::user::User;
use serenity::utils::MessageBuilder;
use std::collections::{HashMap, HashSet};

#[derive(PartialEq, Eq)]
pub struct Game {
    started: bool,
    id: String,
    players: HashSet<User>,
    choices: HashMap<User, Choice>,
}

impl Game {
    pub fn new(id: String) -> Self {
        Game {
            started: false,
            id,
            players: HashSet::new(),
            choices: HashMap::new(),
        }
    }
    pub fn get_player_count(&self) -> usize {
        self.players.len()
    }
    pub fn add_player(&mut self, user: &User) -> bool {
        if !self.players.contains(user) && !self.started {
            self.players.insert(user.clone());
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
    pub fn generate_message(&self) -> String {
        let mut points: HashMap<User, u16> = HashMap::new();
        if let Some(choice) = self.did_all_choose_same() {
            return MessageBuilder::new()
                .push("All players chose ")
                .push(choice_to_emoji(choice))
                .push("\nNo one wins")
                .build();
        } else {
            return MessageBuilder::new()
                .push(
                    self.get_all_interactions()
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
    }
    pub fn start(&mut self) {
        self.started = true;
    }
}
