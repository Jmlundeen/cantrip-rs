use cantrip_api::*;
use cantrip_mechanics::{counter_target, targets};
use cantrip_oracle as oracle;

/// "Counter target red or green spell." (The color words are read through this spell's own text, CR 612, by the target slot.)
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    counter_target(g, item, 0);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::spell_of_either_color::<{ Color::Red as u8 }, { Color::Green as u8 }>()], effect),
    ..oracle::FLASHFREEZE
};
