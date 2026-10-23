use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

/// "Until end of turn, target creature you control becomes an Avatar in addition to its other types and gains flying, first
/// strike, lifelink, and hexproof." (CR 611.2a, 205.1b, 613.1d, 613.1f): one continuing effect of the spell, so the type and
/// the four keywords are one instruction. All of it does nothing if the target is no longer legal (CR 608.2b).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    instruction(g, |g| {
        effects::becomes_subtype_until_end_of_turn(g, item.object, creature, Subtype::Avatar);
        effects::gain_until_end_of_turn(g, item.object, creature, Keyword::Flying);
        effects::gain_until_end_of_turn(g, item.object, creature, Keyword::FirstStrike);
        effects::gain_lifelink_until_end_of_turn(g, item.object, creature);
        effects::gain_hexproof_until_end_of_turn(g, item.object, creature);
    });
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE_YOU_CONTROL], effect),
    ..oracle::ENTER_THE_AVATAR_STATE
};
