use cantrip_api::*;
use cantrip_mechanics::{assimilate, targets, triggers};
use cantrip_mechanics::triggers::THIS_ATTACKS;
use cantrip_oracle as oracle;

/// "Exile target opponent's graveyard" (CR 406.2, 701.13): every card in the graveyard of the targeted opponent, in one
/// move, and the exile is linked to this ability's source, so the attack trigger can assimilate a creature card from it
/// (CR 607.2a). Nothing happens if the player is no longer a legal target (CR 608.2b).
fn exile_the_targets_graveyard(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(player) = targets::player(g, item, 0) else { return Ok(()) };
    let cards = g.state().zone(Zone::graveyard(player)).to_vec();
    let exiled = g.exile_all(&cards);
    // The pile is linked to this source so the attack trigger can assimilate a creature card from it (CR 607.2a); nothing
    // returns it, so this is not an exile until the source leaves (CR 610.3).
    g.link_exiled(&exiled, g.linked(item));
    Ok(())
}

/// "Assimilate a creature card exiled with it": the linked pile the entry trigger filled, one card chosen as this
/// resolves, put onto the battlefield with a +1/+1 counter as a Borg artifact creature (CR 122.6, 205.1b, 614.12).
fn assimilate_one_exiled(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    assimilate::a_creature_card_exiled_with(g, item)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            triggers::THIS_ENTERS,
            exile_the_targets_graveyard,
            "When this creature enters, exile target opponent's graveyard.",
        )
        .targets(&[targets::OPPONENT]),
        triggered(
            THIS_ATTACKS,
            assimilate_one_exiled,
            "Whenever this creature attacks, assimilate a creature card exiled with it.",
        ),
    ],
    ..oracle::ASSIMILATION_ASSESSOR
};
