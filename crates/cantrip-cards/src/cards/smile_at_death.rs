use cantrip_api::*;
use cantrip_mechanics::{effects, targets, triggers::YOUR_UPKEEP};
use cantrip_oracle as oracle;

/// "Creature card with power 2 or less from your graveyard" (CR 115.1, 208.1, 604.3): a creature card in its controller's graveyard
/// whose power as it is there, a characteristic-defining ability's included (Tarmogoyf's), is at most 2.
fn creature_card_with_power_2_or_less_in_your_graveyard(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    targets::card_in_your_graveyard(q, asker, t, &[CardType::Creature]) && matches!(t, Target::Object(id) if q.chars(id).power.is_some_and(|p| p <= 2))
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        YOUR_UPKEEP,
        effects::return_targets_to_battlefield_with_plus_one_counters,
        "At the beginning of your upkeep, return up to two target creature cards with power 2 or less from your graveyard to the battlefield. Put a +1/+1 counter on each of those creatures.",
    )
    .targets(&[target(creature_card_with_power_2_or_less_in_your_graveyard).between(0, 2)])],
    ..oracle::SMILE_AT_DEATH
};
