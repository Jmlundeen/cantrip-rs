use cantrip_api::*;
use cantrip_mechanics::combat::create_attacking_of_your_choice;
use cantrip_mechanics::{tokens, triggers};
use cantrip_oracle as oracle;

/// "For each opponent you attacked this turn" (CR 508.1): each opponent a creature of yours was declared attacking
/// this turn, counted once however many creatures attacked them and whether or not they are still attacking; a
/// planeswalker or battle attacked is not its controller (CR 508.5).
fn opponents_attacked_this_turn(q: &dyn Query, you: PlayerId) -> usize {
    let mut attacked: Vec<PlayerId> = Vec::new();
    for r in q.history().this_turn() {
        if let Event::Attacks { defender: Target::Player(p), .. } = r.event
            && r.active == you
            && p != you
            && !attacked.contains(&p)
        {
            attacked.push(p);
        }
    }
    attacked.len()
}

/// "Whenever you attack, for each opponent you attacked this turn, create a tapped and attacking 2/1 red Klingon
/// Warrior creature token with haste." (CR 508.1, 508.4): the tokens enter tapped and attacking, and their controller
/// chooses what each one attacks; the text names no defender for them.
fn klingons(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let klingon = g.card_id(&tokens::KLINGON);
    let count = opponents_attacked_this_turn(g, item.controller);
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
