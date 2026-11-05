use cantrip_api::*;
use cantrip_mechanics::replacements::DOUBLE_PLUS_ONE_COUNTERS_ON_YOUR_CREATURES;
use cantrip_mechanics::{cost_reduction, effects, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    cost_changes: &[cost_reduction::colored_spells_you_cast_cost_one_less::<{ cost_reduction::color(Color::Green) }>],
    replacements: &[DOUBLE_PLUS_ONE_COUNTERS_ON_YOUR_CREATURES],
    // The targets and their shares, each at least one, are announced as it is activated (CR 601.2d by way of 602.2b);
    // a target illegal as it resolves takes its share with it (the ruling).
    abilities: &[ability(
        cost("{4}{G}{G}, {T}"),
        effects::distribute_plus_one_counters,
        "{4}{G}{G}, {T}: Distribute two +1/+1 counters among one or two target creatures you control.",
    )
    .targets(&[targets::CREATURE_YOU_CONTROL.between(1, 2)])
    .divide(Divide { total: 2, reads: None })],
    ..oracle::THE_EARTH_CRYSTAL
};
