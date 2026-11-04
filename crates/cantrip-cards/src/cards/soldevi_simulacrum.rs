use cantrip_api::*;
use cantrip_mechanics::{cumulative_upkeep, effects};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        cumulative_upkeep!(
            cantrip_mechanics::cumulative_upkeep::Payment::Mana(mana("{1}")),
            "Cumulative upkeep {1} (At the beginning of your upkeep, put an age counter on this permanent, then sacrifice it unless you pay its upkeep cost for each age counter on it.)"
        ),
        ability(
            cost("{1}"),
            effects::this_gets::<1, 0>,
            "{1}: This creature gets +1/+0 until end of turn.",
        ),
    ],
    ..oracle::SOLDEVI_SIMULACRUM
};
