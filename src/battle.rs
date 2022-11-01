use crate::{battleresult::BattleResult, choice::Choice};
use serenity::model::user::User;

pub struct Battle {
    player1: User,
    choice1: Choice,
    player2: User,
    choice2: Choice,
}

impl Battle {
    pub fn new(player1: User, choice1: Choice, player2: User, choice2: Choice) -> Self {
        Battle {
            player1,
            choice1,
            player2,
            choice2,
        }
    }
    pub fn new_ref(player1: &User, choice1: &Choice, player2: &User, choice2: &Choice) -> Self {
        Self::new(player1.clone(), *choice1, player2.clone(), *choice2)
    }
    pub fn battle(&self) -> Option<BattleResult> {
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
