use cantrip_api::*;
use cantrip_mechanics::{exile_target, targets};
use cantrip_oracle as oracle;

/// "Any number of white cards from your hand": the color word read through this spell's words (CR 612).
fn white_card(q: &dyn Query, asker: &Asker, id: ObjectId) -> bool {
    q.chars(id).colors.has(q.words(asker.source).color(Color::White))
}

/// "Target artifact, creature, or enchantment with mana value X or less" (CR 107.3a, 202.3): X is announced before the
/// target is chosen (`late_targets`) and judged again as the spell resolves (CR 608.2b). Before it is announced, every
/// artifact, creature and enchantment is allowed, since some X would allow it.
fn artifact_creature_or_enchantment_with_mana_value_x_or_less(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    targets::artifact_creature_or_enchantment_filter(q, asker, t)
        && t.object().is_some_and(|id| q.announced_x().is_none_or(|x| q.mana_value(id) <= x))
}

/// "Exile target artifact, creature, or enchantment with mana value X or less." Nothing happens if it is no longer a legal
/// target (CR 608.2b).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    exile_target(g, item, 0);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[target(artifact_creature_or_enchantment_with_mana_value_x_or_less)], effect),
    late_targets: true,
    // "As an additional cost to cast this spell, you may exile any number of white cards from your hand. This spell costs {2}
    // less to cast for each card exiled this way." (CR 601.2b, 601.2f, 118.7a)
    additional_cost: Some(Cost { exile_from_hand_for_less: Some(ExileFromHandForLess { filter: white_card, less_each: 2 }), ..Cost::NONE }),
    ..oracle::MARCH_OF_OTHERWORLDLY_LIGHT
};
