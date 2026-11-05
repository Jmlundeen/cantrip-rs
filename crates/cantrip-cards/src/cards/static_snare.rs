use cantrip_api::*;
use cantrip_mechanics::exile_target_until_leaves_effect;
use cantrip_mechanics::targets;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_oracle as oracle;

/// This spell costs {1} less to cast for each attacking creature (CR 601.2f, 506.4): the attackers of every player, generic mana
/// only, and nothing outside combat. A creature that has died or left combat is no longer among `attackers` (CR 506.4).
fn cheaper_per_attacker(q: &dyn Query, _: ObjectId, _: PlayerId, _: Costed, _: &[Target]) -> CostDelta {
    CostDelta::less(q.state().attackers.len() as u32)
}

/// "When this enchantment enters, exile target artifact or creature an opponent controls until this enchantment leaves the
/// battlefield" (CR 610.3, 603.1): the card returns, a new object, when the enchantment leaves; nothing is exiled if it has left
/// before the ability resolves (CR 610.3b).
pub static CARD: CardDef = CardDef {
    own_cost_changes: &[cheaper_per_attacker],
    abilities: &[triggered(
        THIS_ENTERS,
        exile_target_until_leaves_effect,
        "When this enchantment enters, exile target artifact or creature an opponent controls until this enchantment leaves the battlefield.",
    )
    .targets(&[targets::ARTIFACT_OR_CREATURE_AN_OPPONENT_CONTROLS])],
    ..oracle::STATIC_SNARE
};
