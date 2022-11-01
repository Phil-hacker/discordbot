use crate::choice::Choice;
use crate::helper::choice_to_emoji;
use serenity::model::user::User;
use serenity::utils::MessageBuilder;

#[derive(PartialEq, Eq)]
pub struct BattleResult {
    pub winner: User,
    pub winner_choice: Choice,
    pub loser: User,
    pub loser_choice: Choice,
    pub verb: String,
}
impl BattleResult {
    pub fn new(
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
    pub fn to_message(&self) -> String {
        MessageBuilder::new()
            .mention(&self.winner)
            .push(choice_to_emoji(self.winner_choice))
            .push(" ")
            .push(&self.verb)
            .push(" ")
            .push(choice_to_emoji(self.loser_choice))
            .mention(&self.loser)
            .push("\n")
            .build()
    }
}
