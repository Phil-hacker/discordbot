use crate::choice::Choice;
use std::collections::{HashMap, HashSet};
use serenity::model::user::User;

#[derive(PartialEq, Eq)]
pub struct Game {
    pub started: bool,
    pub id: String,
    pub players: HashSet<User>,
    pub choices: HashMap<User, Choice>,
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
}
