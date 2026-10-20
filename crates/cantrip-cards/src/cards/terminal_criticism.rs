use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

/// "Destroy target creature or planeswalker that's blue or red. You gain 1 life." (CR 701.8, 608.2b): with its one target
/// illegal the core does not resolve the spell at all, so no life; with a legal target the life is gained whether or not the
/// destroy did anything (an indestructible target, a regeneration shield), the instructions being independent (CR 608.2c).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    cantrip_mechanics::destroy_target(g, item, 0);
    effects::gain_life(g, item, 1)
}

pub static CARD: CardDef = CardDef { spell: spell(&[targets::creature_or_planeswalker_of_either_color::<{ Color::Blue as u8 }, { Color::Red as u8 }>()], effect), ..oracle::TERMINAL_CRITICISM };
