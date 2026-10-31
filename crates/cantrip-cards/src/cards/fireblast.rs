use cantrip_api::*;
use cantrip_oracle as oracle;
use cantrip_mechanics::{damage_target, targets};

fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    damage_target(g, item, 0, 4);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::ANY], effect),
    ways: &[way(Cost::sacrifice(2, cantrip_mechanics::targets::filters::land_type::<{ cantrip_mechanics::lands::sub(Subtype::Mountain) }>), ZoneKind::Hand, "Sacrifice two Mountains")],
    ..oracle::FIREBLAST
};
