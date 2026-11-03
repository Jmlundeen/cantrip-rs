use cantrip_api::*;
use cantrip_mechanics::combat::create_attacking_of_your_choice;
use cantrip_mechanics::{delayed, triggers};
use cantrip_oracle as oracle;

/// The 2/1 colorless Construct artifact creature token with flying named Ballistic Boulder (CR 111.4).
pub static BALLISTIC_BOULDER: CardDef = CardDef {
    name: "Ballistic Boulder",
    has_cost: false,
    types: TypeSet::of(&[CardType::Artifact, CardType::Creature]),
    subtypes: SubtypeSet::of(&[Subtype::Construct]),
    power: Some(2),
    toughness: Some(1),
    keywords: Keywords::of(&[Keyword::Flying]),
    ..CardDef::BLANK
};

/// "Create a ... token ... that's tapped and attacking" (CR 508.4): its controller chooses what it attacks, a player, a planeswalker
/// or a battle, not necessarily what the other attackers attack (the ruling), asked only where there is a choice. "Sacrifice that
/// token at the beginning of the next end step" (CR 603.7c) is the card's delayed ability 0, about exactly the token made.
fn ballistic_boulder(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let boulder = g.card_id(&BALLISTIC_BOULDER);
    let created = create_attacking_of_your_choice(g, item.controller, item.object, boulder, 1, true)?;
    delayed::at_next_end_step_about(g, item, 0, &created);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        triggers::YOU_ATTACK,
        ballistic_boulder,
        "Whenever you attack, create a 2/1 colorless Construct artifact creature token with flying named Ballistic Boulder that's tapped and attacking. Sacrifice that token at the beginning of the next end step.",
    )],
    // The shared body, with this card's own Oracle wording ("that token", not "them").
    delayed: &[delayed_trigger(delayed::sacrifice_created, "Sacrifice that token at the beginning of the next end step.")],
    ..oracle::FIRE_NAVY_TREBUCHET
};

pub static EXTRA: &[&CardDef] = &[&BALLISTIC_BOULDER];
