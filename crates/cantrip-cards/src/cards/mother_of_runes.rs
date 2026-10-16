use cantrip_api::*;
use cantrip_oracle as oracle;
use cantrip_mechanics::{choose_color, targets};

/// Target creature you control gains protection from the color of your choice until end of turn; the color is
/// chosen as the ability resolves (CR 611.2c).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    let color = choose_color(g, item.controller, item.object)?;
    g.add_effect(item.object, creature, EffectKind::Protection(ColorSet::of(color)), Duration::EndOfTurn);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[ability(TAP, effect, "{T}: Target creature you control gains protection from the color of your choice until end of turn.")
        .targets(&[targets::CREATURE_YOU_CONTROL])],
    ..oracle::MOTHER_OF_RUNES
};
