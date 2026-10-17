use cantrip_api::*;
use cantrip_mechanics::{each_discards_a_card, effects, targets};
use cantrip_oracle as oracle;

/// "+1: Surveil 2." (CR 701.25.)
fn surveil_two(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::surveil(g, item, 2)
}

/// "−1: Any number of target players each discard a card." The targets still legal (CR 608.2b) each choose a card, in turn
/// order from the active player (CR 101.4, the ruling: the next target player in turn order, or the active player if they
/// are one, chooses first), and the chosen cards are discarded at the same time. With none chosen, nothing happens.
fn each_target_discards(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let chosen: Vec<PlayerId> = (0..item.target_count as usize).filter_map(|slot| targets::player(g, item, slot)).collect();
    let players: Vec<PlayerId> = g.state().apnap().filter(|p| chosen.contains(p)).collect();
    each_discards_a_card(g, &players, item.object)?;
    Ok(())
}

/// "−7: Flip five coins. Target opponent skips their next X turns, where X is the number of coins that came up heads."
/// Flips that care only whether they come up heads are won or lost by no one (CR 705.2). Each turn to skip is a
/// replacement effect of one of that player's turns beginning (CR 500.11, 614.10); X of zero skips none. With the
/// opponent no longer a legal target the ability does not resolve (CR 608.2b), and no coin is flipped.
fn skip_turns(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(opponent) = targets::player(g, item, 0) else { return Ok(()) };
    let heads = (0..5).filter(|_| g.flip_coin_for_heads(item.controller)).count();
    g.skip_next_turns(item.object, opponent, heads as u32);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        ability(loyalty(1), surveil_two, "+1: Surveil 2."),
        ability(loyalty(-1), each_target_discards, "\u{2212}1: Any number of target players each discard a card.")
            .targets(&[targets::PLAYER.between(0, TargetSpec::ANY_NUMBER)]),
        // A card with {X} in its cost has X of 0 in the graveyard (the ruling, CR 202.3e), which the filter's mana value reads.
        ability(loyalty(-2), effects::return_first_target_to_battlefield, "\u{2212}2: Return target creature card with mana value 3 or less from your graveyard to the battlefield.")
            .targets(&[targets::CREATURE_CARD_WITH_MANA_VALUE_3_OR_LESS_IN_YOUR_GRAVEYARD]),
        ability(loyalty(-7), skip_turns, "\u{2212}7: Flip five coins. Target opponent skips their next X turns, where X is the number of coins that came up heads.")
            .targets(&[targets::OPPONENT]),
    ],
    ..oracle::RAL_ZAREK_GUEST_LECTURER
};
