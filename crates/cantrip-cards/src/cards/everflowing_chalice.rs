use cantrip_api::*;
use cantrip_mechanics::kicker;
use cantrip_oracle as oracle;

/// "{T}: Add {C} for each charge counter on this artifact", counted as it activates.
fn add_per_charge_counter(g: &mut dyn Game, asker: &Asker, _: Option<Color>) -> Result<(), Suspend> {
    let counters = g.obj(asker.source).counters(CounterKind::Charge);
    if counters > 0 {
        g.add_mana(asker.controller, Mana::C, counters as u8);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    kicker: Some(Kicker { cost: mana(2, 0, 0, 0, 0, 0), multi: true, waterbend: false }),
    enters: Some(kicker::enters_with_counters::<{ CounterKind::Charge as usize }>),
    abilities: &[AbilityDef::Activated(ActivatedDef {
        text: "{T}: Add {C} for each charge counter on this artifact.",
        cost: Cost { tap: true, ..Cost::NONE },
        mana: Some(add_per_charge_counter),
        ..ActivatedDef::BLANK
    })],
    ..oracle::EVERFLOWING_CHALICE
};
