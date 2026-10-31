use cantrip_api::*;
use cantrip_mechanics::{targets, counter_unless_pays};
use cantrip_oracle as oracle;

/// Counter target spell unless its controller pays {1} (CR 118.12).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    counter_unless_pays(g, item, 0, mana("{1}"))?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::SPELL], effect),
    ways: &[way(Cost::return_to_hand(1, cantrip_mechanics::targets::filters::land_type::<{ cantrip_mechanics::lands::sub(Subtype::Island) }>), ZoneKind::Hand, "Return an Island you control to its owner's hand")],
    ..oracle::DAZE
};
