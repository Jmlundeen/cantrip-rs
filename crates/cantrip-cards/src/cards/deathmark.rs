use cantrip_api::*;
use cantrip_mechanics::{destroy_target, targets};
use cantrip_oracle as oracle;

/// "Destroy target green or white creature." (CR 701.8, 115.1): regeneration and indestructible apply, since it does not
/// say otherwise.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    destroy_target(g, item, 0);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::creature_of_either_color::<{ Color::Green as u8 }, { Color::White as u8 }>()], effect),
    ..oracle::DEATHMARK
};
