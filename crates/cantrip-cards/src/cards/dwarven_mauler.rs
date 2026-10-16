use cantrip_api::*;
use cantrip_mechanics::attach::equip_targeting_this_costs_less;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    // "Equip abilities you activate that target this creature cost {2} less to activate." (CR 601.2f, 602.2b)
    cost_changes: &[equip_targeting_this_costs_less::<2>],
    ..oracle::DWARVEN_MAULER
};
