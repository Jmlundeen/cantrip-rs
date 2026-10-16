use cantrip_api::*;
use cantrip_oracle as oracle;
use cantrip_mechanics::{damage_target, targets};

fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    damage_target(g, item, 0, 4);
    Ok(())
}

fn mountain(q: &dyn Query, asker: &Asker, id: ObjectId) -> bool {
    q.chars(id).has_subtype(q.words(asker.source).subtype(Subtype::Mountain))
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::ANY], effect),
    ways: &[way(Cost::sacrifice(2, mountain), ZoneKind::Hand, "Sacrifice two Mountains")],
    ..oracle::FIREBLAST
};
