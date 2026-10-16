use cantrip_api::*;
use cantrip_mechanics::mana;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    enters: Some(mana::sunburst),
    abilities: &[mana_ability(remove_counters(CounterKind::Charge, 1), add_any_color(1), "Remove a charge counter from this artifact: Add one mana of any color.")],
    ..oracle::PENTAD_PRISM
};
