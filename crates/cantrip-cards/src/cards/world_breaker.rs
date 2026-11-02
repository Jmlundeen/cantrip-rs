use cantrip_api::*;
use cantrip_mechanics::{devoid, effects, exile_target, targets, triggers};
use cantrip_oracle as oracle;

/// "When you cast this spell, exile target artifact, enchantment, or land." (CR 115.1).
fn exile_target_artifact_enchantment_land_target_0(
    q: &dyn Query,
    _: &Asker,
    t: Target,
) -> bool {
    matches!(
        t, Target::Object(id) if cantrip_mechanics::targets::permanent(q, id) && ((q
        .chars(id).is(CardType::Artifact) || q.chars(id).is(CardType::Enchantment) || q
        .chars(id).is(CardType::Land)))
    )
}

/// "When you cast this spell, exile target artifact, enchantment, or land." (CR 603.1, 701.13).
fn exile_target_artifact_enchantment_land(
    g: &mut dyn Game,
    item: &StackItem,
) -> Result<(), Suspend> {
    exile_target(g, item, 0);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    colors: devoid::COLORS,
    abilities: &[
        triggered(
            triggers::CAST_THIS_SPELL,
            exile_target_artifact_enchantment_land,
            "When you cast this spell, exile target artifact, enchantment, or land.",
        )
        .targets(&[target(exile_target_artifact_enchantment_land_target_0)]),
        ability(
            cost("{2}{C}").and(sacrifice(1, targets::filters::land)),
            effects::return_this_to_hand,
            "{2}{C}, Sacrifice a land: Return this card from your graveyard to your hand.",
        )
        .from(ZoneKind::Graveyard),
    ],
    ..oracle::WORLD_BREAKER
};
