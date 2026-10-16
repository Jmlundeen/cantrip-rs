use cantrip_api::*;
use cantrip_mechanics::{pump, targets};
use cantrip_oracle as oracle;

/// Target creature gets +X/+0 and gains trample until end of turn, X being what was paid.
fn boost(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(creature) = targets::object(g, item, 0) {
        pump(g, item.object, creature, item.x as i16, 0);
        cantrip_mechanics::trample_until_end_of_turn(g, item.object, creature);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add(Mana::C), "{T}: Add {C}."),
        ability(cost("{X}{R}{G}, {T}"), boost, "{X}{R}{G}, {T}: Target creature gets +X/+0 and gains trample until end of turn.")
            .targets(&[targets::CREATURE]),
    ],
    ..oracle::KESSIG_WOLF_RUN
};
