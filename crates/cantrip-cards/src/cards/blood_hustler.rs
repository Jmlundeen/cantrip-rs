use cantrip_api::*;
use cantrip_mechanics::effects::plus_one_counter_on_this;
use cantrip_mechanics::{targets, triggers};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(triggers::YOU_COMMIT_A_CRIME, plus_one_counter_on_this, "Whenever you commit a crime, put a +1/+1 counter on this creature. This ability triggers only once each turn. (Targeting opponents, anything they control, and/or cards in their graveyards is a crime.)")
            .triggers_each_turn(1),
        ability(cost("{3}{B}"), cantrip_mechanics::effects::target_player_loses_1_life_you_gain_1, "{3}{B}: Target opponent loses 1 life and you gain 1 life.").targets(&[targets::OPPONENT]),
    ],
    ..oracle::BLOOD_HUSTLER
};
