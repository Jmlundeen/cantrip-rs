use cantrip_api::*;
use cantrip_mechanics::{companion, effects, targets};
use cantrip_oracle as oracle;

/// "Abilities you activate that aren't mana abilities cost {2} less to activate. This effect can't reduce the mana in
/// that cost to less than one mana" (CR 601.2f, 118.7): any activated ability its controller activates, from whatever zone,
/// that isn't a mana ability (CR 605.1a). The core asks the changes of mana abilities too (CR 605.3), so the exception is
/// this card's to make; a special action is no ability (CR 116.1): `Costed::Ability` alone.
fn two_less(q: &dyn Query, source: ObjectId, player: PlayerId, costed: Costed, _: &[Target]) -> CostDelta {
    let Costed::Ability(_, ability, _) = costed else { return CostDelta::NONE };
    if player == q.controller(source) && q.activated_ability(ability).is_some_and(|d| !d.is_mana_ability()) { CostDelta::less_to_one(2) } else { CostDelta::NONE }
}

/// "Companion - Each permanent card in your starting deck has an activated ability" (CR 702.139a, 602.1): a mana ability is
/// one, a land's by its basic land types too (CR 305.6, the rulings); a special action is not (CR 116.1).
fn permanents_with_activated_abilities(q: &dyn Query, deck: &StartingDeck) -> bool {
    companion::each_permanent_card(q, deck, |card| {
        q.chars(card).abilities().any(|a| q.activated_ability(a).is_some_and(|d| !d.special_action))
    })
}

pub static CARD: CardDef = CardDef {
    companion: Some(permanents_with_activated_abilities),
    abilities: &[ability(cost("{1}, {T}"), effects::first_target_cant_block_this_turn, "{1}, {T}: Target creature can't block this turn.")
        .targets(&[targets::CREATURE])],
    cost_changes: &[two_less],
    ..oracle::ZIRDA_THE_DAWNWAKER
};
