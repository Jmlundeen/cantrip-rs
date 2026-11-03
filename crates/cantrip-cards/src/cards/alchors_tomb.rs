use cantrip_api::*;
use cantrip_mechanics::{choose_color, targets};
use cantrip_oracle as oracle;

/// The target becomes the chosen color indefinitely (CR 608.2d, 611.2a, 613.1e).
fn change_color(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(permanent) = targets::object(g, item, 0) else { return Ok(()) };
    let color = choose_color(g, item.controller, item.object)?;
    g.add_effect(item.object, permanent, EffectKind::SetColors(ColorSet::of(color)), Duration::Indefinite);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[ability(cost("{2}").and(TAP), change_color, "{2}, {T}: Target permanent you control becomes the color of your choice.")
        .targets(&[targets::PERMANENT_YOU_CONTROL])],
    ..oracle::ALCHORS_TOMB
};
