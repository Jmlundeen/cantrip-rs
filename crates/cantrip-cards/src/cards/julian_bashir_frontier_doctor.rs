use cantrip_api::*;
use cantrip_mechanics::seek_out::seek_out;
use cantrip_mechanics::{effects, targets, triggers};
use cantrip_oracle as oracle;

/// "Target creature card with mana value X or less from your graveyard, where X is Julian Bashir's power" (CR 115.1,
/// 202.3): a quality of the target alone, X read from its source as the target is chosen and again as the ability
/// resolves (CR 608.2b); a card in a graveyard with {X} in its cost has X of 0 (CR 202.3e), which `Query::mana_value`
/// reads.
fn creature_card_in_your_graveyard_with_mana_value_at_most_his_power(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    targets::card_in_your_graveyard(q, asker, t, &[CardType::Creature])
        && t.object().is_some_and(|card| {
            // Compared signed (CR 107.1b): with negative power no mana value, 0 included, is at most X.
            i64::from(q.mana_value(card)) <= i64::from(q.chars(asker.source).power.unwrap_or_default())
        })
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        seek_out(
            effects::plus_one_counter_on_this,
            &[],
            "Seek out — Whenever you draw your second card each turn, put a +1/+1 counter on Julian Bashir.",
        ),
        triggered(
            triggers::THIS_ATTACKS,
            effects::return_first_target_to_battlefield,
            "Whenever Julian Bashir attacks, return target creature card with mana value X or less from your graveyard to the battlefield, where X is Julian Bashir's power.",
        )
        .targets(&[target(creature_card_in_your_graveyard_with_mana_value_at_most_his_power)]),
    ],
    ..oracle::JULIAN_BASHIR_FRONTIER_DOCTOR
};
