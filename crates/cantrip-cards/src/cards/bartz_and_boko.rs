use cantrip_api::*;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_mechanics::{affinity, fight, lands, targets};
use cantrip_oracle as oracle;

/// "Each other Bird you control deals damage equal to its power to target creature an opponent controls." (CR 120.3, 609.7,
/// 608.2b): every Bird the ability's controller controls as it resolves but Bartz and Boko itself (a Bird too: "other"), by the
/// layers (a changeling is one), each the source of its own damage, so its abilities (lifelink, deathtouch) and colors count; all
/// dealt at the same time (CR 120.2), the power read now (CR 608.2c) and a Bird with none, or less, dealing none (CR 120.8). One target however many
/// Birds there are (the ruling). If the creature is no longer a legal target as it resolves, nothing is dealt.
fn birds_deal_damage(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(victim) = targets::object(g, item, 0) else { return Ok(()) };
    let bird = g.words(item.object).subtype(Subtype::Bird);
    let hits: Vec<(ObjectId, Target, u32)> = g
        .permanents_of(item.controller)
        .into_iter()
        .filter(|&id| id != item.object && g.chars(id).has_subtype(bird))
        .map(|id| (id, Target::Object(victim), fight::damage_equal_to_power(&*g, id)))
        .collect();
    g.damage_together(&hits);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    own_cost_changes: &[affinity::for_creature_type::<{ lands::sub(Subtype::Bird) }>],
    abilities: &[triggered(
        THIS_ENTERS,
        birds_deal_damage,
        "When Bartz and Boko enters, each other Bird you control deals damage equal to its power to target creature an opponent controls.",
    )
    .targets(&[targets::CREATURE_AN_OPPONENT_CONTROLS])],
    ..oracle::BARTZ_AND_BOKO
};
