use cantrip_api::*;
use cantrip_mechanics::{lands, targets};
use cantrip_oracle as oracle;

/// "This land deals 2 damage to each opponent."
fn damage_opponents(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    cantrip_mechanics::effects::damage_each_opponent(g, item, 2);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add(Mana::C), "{T}: Add {C}."),
        mana_ability(TAP.and(life(1)), add(Mana::R), "{T}, Pay 1 life: Add {R}."),
        ability(
            cost("{2}{R}{R}, {T}").and(sacrifice(1, targets::filters::land_type::<{ lands::sub(Subtype::Desert) }>)),
            damage_opponents,
            "{2}{R}{R}, {T}, Sacrifice a Desert: This land deals 2 damage to each opponent.",
        ),
    ],
    ..oracle::RAMUNAP_RUINS
};
