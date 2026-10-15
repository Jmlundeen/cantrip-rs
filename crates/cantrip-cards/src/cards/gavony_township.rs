use cantrip_api::*;
use cantrip_mechanics::mana;
use cantrip_oracle as oracle;

/// Put a +1/+1 counter on each creature you control.
fn counters(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    cantrip_mechanics::effects::counters_on_creatures_you_control(g, item.controller, CounterKind::PlusOne, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana::tap_for(mana::add_c, Mana::C, "{T}: Add {C}."),
        AbilityDef::Activated(ActivatedDef {
            text: "{2}{G}{W}, {T}: Put a +1/+1 counter on each creature you control.",
            cost: Cost { mana: mana(2, 1, 0, 0, 0, 1), tap: true, ..Cost::NONE },
            effect: counters,
            ..ActivatedDef::BLANK
        }),
    ],
    ..oracle::GAVONY_TOWNSHIP
};
