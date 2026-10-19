use cantrip_api::*;
use cantrip_mechanics::statics;
use cantrip_oracle as oracle;
/// Generic reduction equals the greatest current power (CR 601.2f, 118.7a).
fn reduction(q: &dyn Query, _: ObjectId, p: PlayerId, costed: Costed, _: &[Target]) -> CostDelta {
 if !matches!(costed, Costed::Spell(_)) { return CostDelta::NONE; }
 let n = q.permanents_of(p).into_iter().filter(|&id| q.chars(id).is(CardType::Creature)).map(|id| q.chars(id).power.unwrap_or(0)).max().unwrap_or(0).max(0);
 CostDelta::less(n.min(255) as u8)
}
pub static CARD: CardDef = CardDef {
 own_cost_changes: &[reduction],
 statics: &[static_ability(Layer::Ability, statics::other_creatures_you_control, statics::have_trample, "Other creatures you control have trample.")],
 ..oracle::GHALTA_THE_UNSTOPPABLE
};
