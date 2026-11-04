use cantrip_api::*;
use cantrip_mechanics::{connive, effects, targets};
use cantrip_oracle as oracle;

/// "Target player draws a card, then up to one target creature you control connives." Each target is judged on its own as the
/// spell resolves (CR 608.2b), so a player or creature that became illegal is left alone and the other still happens; the spell
/// is countered only if every target is illegal. The draw and the connive are events, so "then" (CR 608.2c) needs no sentence
/// break. The creature's controller, whoever the target player was, draws and discards (CR 701.50a).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::target_player_draws(g, item, 0, 1);
    let Some(creature) = targets::object(g, item, 1) else { return Ok(()) };
    connive(g, creature, item.object)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(
        &[
            targets::PLAYER.labelled("Target player draws a card"),
            targets::CREATURE_YOU_CONTROL.optional().labelled("Up to one target creature you control connives"),
        ],
        effect,
    ),
    ..oracle::UNSTABLE_EXPERIMENT
};
