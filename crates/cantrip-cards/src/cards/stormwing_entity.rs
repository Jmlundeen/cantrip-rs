use cantrip_api::*;
use cantrip_mechanics::triggers::{self, THIS_ENTERS};
use cantrip_mechanics::effects;
use cantrip_oracle as oracle;

/// "This spell costs {2}{U} less to cast if you've cast an instant or sorcery spell this turn." (CR 601.2f.) Asked of
/// the event history, so a countered spell counts, and applied once however many were cast.
fn less_after_an_instant_or_sorcery(q: &dyn Query, _: ObjectId, player: PlayerId, _: Costed, _: &[Target]) -> CostDelta {
    if q.history().casts_by(player).any(|cast| cast.is(CardType::Instant) || cast.is(CardType::Sorcery)) {
        CostDelta::less_mana(mana("{2}{U}"))
    } else {
        CostDelta::NONE
    }
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggers::PROWESS, triggered(THIS_ENTERS, effects::scry_two, "When this creature enters, scry 2.")],
    own_cost_changes: &[less_after_an_instant_or_sorcery],
    ..oracle::STORMWING_ENTITY
};
