use cantrip_api::*;
use cantrip_mechanics::hand::discard_chosen_nonland_from_revealed_hand;
use cantrip_mechanics::{effects, evoke, spent, targets, triggers::THIS_ENTERS};
use cantrip_oracle as oracle;

/// "Target opponent reveals their hand. You choose a nonland card from it. That player discards that card." Asked even
/// when nothing in the hand is eligible: looking at the hand is part of the ability.
fn discard_nonland(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(opponent) = item.targets[0].player() else { return Ok(()) };
    discard_chosen_nonland_from_revealed_hand(g, item, opponent, Purpose::Discard)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, effects::return_first_target_to_hand, "When this creature enters, if {U}{U} was spent to cast it, return up to one other target nonland permanent to its owner's hand.")
            .targets(&[up_to_one(targets::other_nonland_permanent_filter)])
            .condition(spent::two::<{ Mana::U as u8 }>),
        triggered(THIS_ENTERS, discard_nonland, "When this creature enters, if {B}{B} was spent to cast it, target opponent reveals their hand. You choose a nonland card from it. That player discards that card.")
            .targets(&[targets::OPPONENT])
            .condition(spent::two::<{ Mana::B as u8 }>),
        evoke::sacrifice_trigger("When Deceit enters, if its evoke cost was paid, its controller sacrifices it."),
    ],
    ways: &[evoke::way(Cost::mana(mana("{U/B}{U/B}")), "Evoke {U/B}{U/B}")],
    ..oracle::DECEIT
};
