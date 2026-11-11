use cantrip_api::*;
use cantrip_mechanics::{effects, targets, triggers::LANDFALL};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(LANDFALL, effects::charge_counters_on_self::<1>, "Landfall \u{2014} Whenever a land you control enters, put a charge counter on this artifact."),
        mana_ability(TAP, add_any_color(1), "{T}: Add one mana of any color."),
        mana_ability(
            TAP.and(remove_counters(CounterKind::Charge, 2)),
            add_n(Mana::C, 2),
            "{T}, Remove two charge counters from this artifact: Add {C}{C}.",
        ),
        ability(TAP.and(remove_counters(CounterKind::Charge, 3)), effects::damage_first_target::<3>, "{T}, Remove three charge counters from this artifact: It deals 3 damage to any target.")
            .targets(&[targets::ANY]),
    ],
    ..oracle::WEATHER_MAKER
};
