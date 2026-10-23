use cantrip_api::*;
use cantrip_mechanics::{counter_unless_pays, targets};
use cantrip_oracle as oracle;

/// "Counter target spell unless its controller pays {3}" (CR 118.12).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    counter_unless_pays(g, item, 0, mana("{3}"))?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::SPELL], effect),
    own_cost_changes: &[targets::less_if_it_targets_color::<2, { Color::Blue as u8 }>],
    ..oracle::MYSTICAL_DISPUTE
};
