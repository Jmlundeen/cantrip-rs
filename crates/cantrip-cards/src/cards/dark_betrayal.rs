use cantrip_api::*;
use cantrip_mechanics::{destroy_target, targets};
use cantrip_oracle as oracle;

/// "Destroy target black creature." (CR 701.8): regeneration and indestructible apply, since it does not say otherwise.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    destroy_target(g, item, 0);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::creature_of_color::<{ Color::Black as u8 }>()], effect),
    ..oracle::DARK_BETRAYAL
};
