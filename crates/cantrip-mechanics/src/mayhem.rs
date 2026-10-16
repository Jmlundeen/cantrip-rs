//! Mayhem [cost] (CR 702.187a): "You may cast this card from your graveyard if you discarded it this turn. Timing rules
//! still apply." Carnage, Crimson Chaos prints it; Norman Osborn's back face (Green Goblin) gives it to every nonland
//! card in its controller's graveyard (CR 113.6).
//!
//! A card with mayhem lists `way(cost, text)` among its `ways`. A card that grants it lists `GRANTED` among its
//! `granted_ways`, the cost being each card's own mana cost.

use cantrip_api::*;

/// Whether the card was discarded this turn: asked of the event history.
pub fn discarded_this_turn(q: &dyn Query, asker: &Asker) -> bool {
    q.history().discarded_this_turn(asker.source)
}

/// The alternative cost of mayhem: cast from the graveyard for `cost`, if it was discarded this turn.
pub const fn way(cost: ManaCost, text: &'static str) -> WayDef {
    cantrip_api::way(Cost::mana(cost), ZoneKind::Graveyard, text)
        .condition(discarded_this_turn)
}

/// "Each nonland card in your graveyard has mayhem. The mayhem cost is equal to its mana cost": for the cards in the
/// granter's controller's graveyard that have a mana cost, cast by that controller, if discarded this turn.
fn granted_cost(q: &dyn Query, asker: &Asker, card: ObjectId, caster: PlayerId) -> Option<Cost> {
    let obj = q.obj(card);
    let def = q.card(obj.card);
    if caster != asker.controller || obj.owner != asker.controller || def.is(CardType::Land) || !def.has_cost {
        return None;
    }
    if !q.history().discarded_this_turn(card) {
        return None;
    }
    Some(Cost::mana(def.cost))
}

pub const GRANTED: GrantedWay = GrantedWay { text: "Mayhem", cost: granted_cost, ..GrantedWay::CAST };
