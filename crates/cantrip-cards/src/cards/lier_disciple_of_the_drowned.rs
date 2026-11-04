use cantrip_api::*;
use cantrip_mechanics::graveyards::{card_is, instant_or_sorcery};
use cantrip_mechanics::statics;
use cantrip_oracle as oracle;

/// "Each instant and sorcery card in your graveyard has flashback. The flashback cost is equal to that card's mana cost."
/// (CR 702.34a). The card is judged as it is in the graveyard (CR 715.4, 709.4), the cost is that of the face cast (CR 709.3a).
fn flashback_for_its_mana_cost(q: &dyn Query, asker: &Asker, card: ObjectId, player: PlayerId) -> Option<Cost> {
    let def = q.card(q.chars(card).card);
    (player == asker.controller && card_is(q, card, instant_or_sorcery) && def.has_cost).then(|| Cost::mana(def.cost))
}

pub static CARD: CardDef = CardDef {
    statics: &[static_ability(Layer::Ability, statics::spells, statics::cant_be_countered, "Spells can't be countered.").everywhere()],
    granted_ways: &[GrantedWay { text: "Flashback", flashback: true, cost: flashback_for_its_mana_cost, ..GrantedWay::CAST }],
    ..oracle::LIER_DISCIPLE_OF_THE_DROWNED
};
