use cantrip_api::*;
use cantrip_mechanics::statics;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    // "Creatures can't attack you unless their controller pays {2} for each creature they control that's attacking you."
    // (CR 508.1d, 508.1h): a cost for each attack on its controller, added to any other (the rulings: two Propagandas cost
    // {4}), paid as attackers are declared; a creature attacking a planeswalker is not attacking "you".
    attack_costs: &[statics::attacking_you_costs::<2>],
    ..oracle::PROPAGANDA
};
