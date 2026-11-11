use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        ability(cost("{2}").and(TAP), effects::charge_counters_on_target::<2>, "{2}, {T}: Put two charge counters on target artifact.")
            .targets(&[targets::ARTIFACT]),
        ability(cost("{3}").and(TAP), effects::target_creature_cant_be_blocked_this_turn, "{3}, {T}: Target creature you control can't be blocked this turn.")
            .targets(&[targets::CREATURE_YOU_CONTROL]),
        ability(cost("{4}").and(TAP), effects::draw_a_card_then_may_put_a_land_tapped, "{4}, {T}: Draw a card. You may put a land card from your hand onto the battlefield tapped."),
    ],
    ..oracle::JEFFERIES_TUBE
};
