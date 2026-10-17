use cantrip_api::*;
use cantrip_mechanics::{damage_target, history, mana, targets};
use cantrip_oracle as oracle;

/// Kazuul's Fury deals damage equal to the sacrificed creature's power to any target; a power of 0 or less deals none.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let power = history::sacrificed_creatures_power(g, item.object);
    if power > 0 {
        damage_target(g, item, 0, power.min(u16::MAX as i32) as u16);
    }
    Ok(())
}

pub static BACK: CardDef = CardDef {
    abilities: &[mana_ability(TAP, add(Mana::R), "{T}: Add {R}.")],
    enters: Some(mana::enters_tapped),
    ..oracle::KAZUULS_CLIFFS
};

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::ANY], effect),
    // As an additional cost to cast this spell, sacrifice a creature (CR 118.8).
    additional_cost: Some(Cost::sacrifice(1, targets::filters::creature)),
    back: Some(&BACK),
    ..oracle::KAZUULS_FURY
};
