use cantrip_api::*;
use cantrip_mechanics::combat::create_attacking_of_your_choice;
use cantrip_mechanics::{battlefront, tokens, triggers};
use cantrip_oracle as oracle;

/// "Whenever you attack, for each opponent you attacked this turn, create a tapped and attacking 2/1 red Klingon
/// Warrior creature token with haste." (CR 508.1, 508.4): the tokens enter tapped and attacking, and their controller
/// chooses what each one attacks; the text names no defender for them.
fn klingons(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let klingon = g.card_id(&tokens::KLINGON);
    let count = battlefront::opponents_attacked_this_turn(g, item.controller) as usize;
    create_attacking_of_your_choice(g, item.controller, item.object, klingon, count, true)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        triggers::YOU_ATTACK,
        klingons,
        "Battlefront — Whenever you attack, for each opponent you attacked this turn, create a tapped and attacking 2/1 red Klingon Warrior creature token with haste.",
    )],
    ..oracle::RITE_OF_ASCENSION
};

pub static EXTRA: &[&CardDef] = &[&tokens::KLINGON];
