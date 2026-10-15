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
    spell: Some(SpellDef { targets: &[targets::ANY], effect }),
    ways: &[WayDef {
        text: "Sacrifice two Mountains",
        cost: Cost::sacrifice(2, mountain),
        from: ZoneKind::Hand,
        exile_after: false,
        condition: None,
        instant_speed: false,
    }],
    ..oracle::FIREBLAST
};
