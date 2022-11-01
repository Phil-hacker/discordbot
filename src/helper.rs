use crate::choice::Choice;
use serenity::model::prelude::ReactionType;

pub fn choice_to_emoji(c: Choice) -> ReactionType {
    match c {
        Choice::Rock => ReactionType::Unicode("🪨".to_string()),
        Choice::Paper => ReactionType::Unicode("📄".to_string()),
        Choice::Scissors => ReactionType::Unicode("✂️".to_string()),
        Choice::Lizard => ReactionType::Unicode("🦎".to_string()),
        Choice::Spock => ReactionType::Unicode("🖖".to_string()),
    }
}
