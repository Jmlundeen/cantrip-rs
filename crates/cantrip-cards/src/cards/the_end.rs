use cantrip_api::*;
use cantrip_mechanics::{library, targets};
use cantrip_oracle as oracle;

/// "This spell costs {2} less to cast if your life total is 5 or less" (CR 601.2f, 118.7): `player` is the one casting it,
/// and only the cost of casting it, not of an ability. It changes only the total cost: the mana value stays 4.
fn cheaper_at_five_life_or_less(q: &dyn Query, _: ObjectId, player: PlayerId, costed: Costed, _: &[Target]) -> CostDelta {
    if matches!(costed, Costed::Spell(_)) && q.state().player(player).life <= 5 {
        CostDelta::less(2)
    } else {
        CostDelta::NONE
    }
}

/// "Exile target creature or planeswalker. Search its controller's graveyard, hand, and library for any number of cards with
/// the same name as that permanent and exile them. That player shuffles, then draws a card for each card exiled from their
/// hand this way." With the target illegal as it resolves none of it happens (CR 608.2b). The controller is the one the
/// permanent had as it left (CR 608.2h). "The same name as that permanent" is the name it has as it exists on the battlefield
/// (a copy has the name it copies, a face-down permanent has none, a transformed one its back face's: `object_names`), read
/// before it is exiled, since exiling it makes a new object (CR 400.7) that is only the card. So a token's name finds the
/// cards of that name too, and a back face of a double-faced card finds nothing (the cards elsewhere have only their front
/// face's name).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(target) = targets::object(g, item, 0) else { return Ok(()) };
    let controller = g.controller(target);
    let names = library::object_names(g, target);
    g.exile(target);
    library::exile_same_name_from_graveyard_hand_and_library(g, item.controller, controller, item.object, &names)
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE_OR_PLANESWALKER], effect),
    own_cost_changes: &[cheaper_at_five_life_or_less],
    ..oracle::THE_END
};
