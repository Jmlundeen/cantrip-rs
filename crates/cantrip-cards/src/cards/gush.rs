use cantrip_api::*;
use cantrip_oracle as oracle;

fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.draw(item.controller, 2);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[], effect),
    ways: &[way(Cost::return_to_hand(2, cantrip_mechanics::targets::filters::land_type::<{ cantrip_mechanics::lands::sub(Subtype::Island) }>), ZoneKind::Hand, "Return two Islands you control to their owner's hand")],
    ..oracle::GUSH
};
