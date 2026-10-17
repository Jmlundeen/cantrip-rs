use cantrip_api::*;
use cantrip_mechanics::effects::plus_one_counter_on_this;
use cantrip_mechanics::effects::enters_with_x_counters;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    enters: Some(enters_with_x_counters),
    abilities: &[
        mana_ability(remove_counters(CounterKind::PlusOne, 1), add_n(Mana::C, 2).only_for(ManaRestriction::ABILITIES), "Remove a +1/+1 counter from this creature: Add {C}{C}. Spend this mana only to activate abilities."),
        ability(cost("{1}, {T}"), plus_one_counter_on_this, "{1}, {T}: Put a +1/+1 counter on this creature."),
    ],
    ..oracle::CRYPTIC_TRILOBITE
};
