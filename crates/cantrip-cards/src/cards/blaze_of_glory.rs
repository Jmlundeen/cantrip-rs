use cantrip_api::*;
use cantrip_mechanics::{combat, targets};
use cantrip_oracle as oracle;

/// "Target creature defending player controls can block any number of creatures this turn. It blocks each attacking creature
/// this turn if able." The first sentence lifts the creature's limit on how many it may block (CR 509.1a, 509.1g) and nothing
/// else: restrictions on what it may block still apply. The second is a requirement (CR 509.1c) that has the creature block
/// each attacker it is able to.
fn blaze(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    g.add_effect(item.object, creature, EffectKind::BlockCapacity(BlockCapacity::AnyNumber), Duration::EndOfTurn);
    g.end_sentence();
    g.add_effect(item.object, creature, EffectKind::MustBlockEach, Duration::EndOfTurn);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[target(targets::spell_creature_defending_player_controls)], blaze),
    cast_only_if: Some(combat::cast_only_before_blockers),
    ..oracle::BLAZE_OF_GLORY
};
