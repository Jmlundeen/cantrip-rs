use cantrip_api::*;
use cantrip_mechanics::{effects, targets, triggers::BEGINNING_OF_YOUR_COMBAT};
use cantrip_oracle as oracle;

/// "Up to one target creature gains first strike and vigilance until end of turn" (CR 611.2a, 702.7, 702.20): one sentence's
/// effects. Nothing if no target was chosen, or it is no longer legal when this resolves (CR 608.2b).
fn gain_first_strike_and_vigilance(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    instruction(g, |g| {
        effects::gain_until_end_of_turn(g, item.object, creature, Keyword::FirstStrike);
        effects::gain_until_end_of_turn(g, item.object, creature, Keyword::Vigilance);
    });
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        BEGINNING_OF_YOUR_COMBAT,
        gain_first_strike_and_vigilance,
        "At the beginning of combat on your turn, up to one target creature gains first strike and vigilance until end of turn.",
    )
    .targets(&[targets::CREATURE.optional()])],
    ..oracle::WILD_PACK_SQUAD
};
