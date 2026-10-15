use cantrip_api::*;
use cantrip_mechanics::mana;
use cantrip_oracle as oracle;

/// "This land enters tapped unless you control a basic land" (CR 614.1c): read on the battlefield as it is while the
/// land enters, so the land itself is not counted.
fn enters(g: &mut dyn Game, entering: &mut Entering) -> Result<(), Suspend> {
    let controls_basic = g
        .permanents_of(entering.controller)
        .into_iter()
        .any(|id| g.obj(id).became.is_none() && g.chars(id).is(CardType::Land) && g.chars(id).supertypes.basic);
    if !controls_basic {
        entering.tapped = true;
    }
    Ok(())
}

/// Put a +1/+1 counter on each creature you control.
fn counters(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    cantrip_mechanics::effects::counters_on_creatures_you_control(g, item.controller, CounterKind::PlusOne, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    enters: Some(enters),
    abilities: &[
        mana::tap_for(mana::add_w, Mana::W, "{T}: Add {W}."),
        AbilityDef::Activated(ActivatedDef {
            text: "{3}{W}, {T}: Put a +1/+1 counter on each creature you control.",
            cost: Cost { mana: mana(3, 1, 0, 0, 0, 0), tap: true, ..Cost::NONE },
            effect: counters,
            ..ActivatedDef::BLANK
        }),
    ],
    ..oracle::ABANDONED_AIR_TEMPLE
};
