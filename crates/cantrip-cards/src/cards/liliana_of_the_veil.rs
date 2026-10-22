use cantrip_api::*;
use cantrip_oracle as oracle;
use cantrip_mechanics::{choose, each_discards_a_card, targets};

/// +1: each player discards a card, chosen in turn order from the active player (CR 101.4), discarded together.
fn plus_one(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let players: Vec<PlayerId> = g.state().apnap().collect();
    each_discards_a_card(g, &players, item.object)?;
    Ok(())
}

/// -2: target player sacrifices a creature, their choice when they have more than one.
fn minus_two(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(victim) = item.target(0).player() else { return Ok(()) };
    cantrip_mechanics::sacrifice_chosen(g, victim, item.object, 1, |c| c.is(CardType::Creature)).map(drop)
}

/// -6: separate all permanents target player controls into two piles (the ability's controller chooses the first;
/// the rest are the second, CR 700.3), and that player sacrifices the pile of their choice.
fn minus_six(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(victim) = item.target(0).player() else { return Ok(()) };
    let permanents = g.permanents_of(victim);
    let (first, second) = choose::separate(g, Some(item.controller), item.object, permanents)?;
    let (pile, _) = choose::pile(g, victim, item.object, first, second)?;
    g.sacrifice(&pile);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        ability(loyalty(1), plus_one, "+1: Each player discards a card."),
        ability(loyalty(-2), minus_two, "\u{2212}2: Target player sacrifices a creature.")
            .targets(&[targets::PLAYER]),
        ability(loyalty(-6), minus_six, "\u{2212}6: Separate all permanents target player controls into two piles. That player sacrifices all permanents in the pile of their choice.")
            .targets(&[targets::PLAYER]),
    ],
    ..oracle::LILIANA_OF_THE_VEIL
};
