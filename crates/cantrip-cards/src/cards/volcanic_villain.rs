use cantrip_api::*;
use cantrip_mechanics::{effects, power_up};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[power_up::ability(
        cost("{5}{R}"),
        effects::two_plus_one_counters_on_this,
        "Power-up \u{2014} {5}{R}: Put two +1/+1 counters on this creature. (Activate each power-up ability only once. Reduce the cost by its mana cost if it entered this turn.)",
    )],
    ..oracle::VOLCANIC_VILLAIN
};
