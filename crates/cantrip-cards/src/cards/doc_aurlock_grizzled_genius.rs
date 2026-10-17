use cantrip_api::*;
use cantrip_mechanics::zone_cast_from;
use cantrip_oracle as oracle;

/// "Spells you cast from your graveyard or from exile cost {2} less to cast" (CR 601.2f, 118.7a): the zone the card is in as
/// its cost is worked out, or the one it was cast from once it is on the stack; generic mana only, never below {0}. Spells
/// of other players, and spells cast from anywhere else, are not affected.
fn graveyard_and_exile_spells_cost_two_less(q: &dyn Query, source: ObjectId, player: PlayerId, costed: Costed, _: &[Target]) -> CostDelta {
    let Costed::Spell(spell) = costed else { return CostDelta::NONE };
    if player == q.controller(source) && matches!(zone_cast_from(q, spell), Some(ZoneKind::Graveyard | ZoneKind::Exile)) {
        CostDelta::less(2)
    } else {
        CostDelta::NONE
    }
}

/// "Plotting cards from your hand costs {2} less" (CR 116.2k, 702.170a, 118.7a): plot's special action, taken by this
/// permanent's controller, of a card in their hand (a card plotted from elsewhere, as Fblthp, Lost on the Range plots from the
/// top of a library, is not); generic mana of the plot cost only (its rulings). Becoming plotted any other way pays nothing
/// to reduce.
fn plotting_costs_two_less(q: &dyn Query, source: ObjectId, player: PlayerId, costed: Costed, _: &[Target]) -> CostDelta {
    let Costed::SpecialAction(card, ability) = costed else { return CostDelta::NONE };
    let from_your_hand = q.obj(card).zone == Zone::hand(player);
    if player == q.controller(source) && from_your_hand && q.activated_ability(ability).is_some_and(|d| d.plot) {
        CostDelta::less(2)
    } else {
        CostDelta::NONE
    }
}

pub static CARD: CardDef = CardDef {
    cost_changes: &[graveyard_and_exile_spells_cost_two_less, plotting_costs_two_less],
    ..oracle::DOC_AURLOCK_GRIZZLED_GENIUS
};
