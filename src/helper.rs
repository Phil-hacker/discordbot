use crate::choice::Choice;
use serenity::model::prelude::ReactionType;

pub fn get_choice_from_char(c: char) -> Option<Choice> {
    match c {
        'r' => Some(Choice::Rock),
        'p' => Some(Choice::Paper),
        's' => Some(Choice::Scissors),
        'l' => Some(Choice::Lizard),
        'S' => Some(Choice::Spock),
        _ => None,
    }
}

pub fn choice_to_emoji(c: Choice) -> ReactionType {
    match c {
        Choice::Rock => ReactionType::Unicode("🪨".to_string()),
        Choice::Paper => ReactionType::Unicode("📄".to_string()),
        Choice::Scissors => ReactionType::Unicode("✂️".to_string()),
        Choice::Lizard => ReactionType::Unicode("🦎".to_string()),
        Choice::Spock => ReactionType::Unicode("🖖".to_string()),
    }
}
