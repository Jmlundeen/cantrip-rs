use cantrip_api::*;
use cantrip_mechanics::{counter_target, targets};
use cantrip_oracle as oracle;

/// Counter the targeted green spell if it remains legal (CR 608.2b, 701.6a).
fn counter(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    counter_target(g, item, 0);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[ability(cost("{B}{B}"), counter, "{B}{B}: Counter target green spell.")
        .targets(&[targets::spell_of_either_color::<{ Color::Green as u8 }, { Color::Green as u8 }>()])],
    ..oracle::DEATHGRIP
};
