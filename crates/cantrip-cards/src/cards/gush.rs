use cantrip_api::*;
use cantrip_oracle as oracle;

fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.draw(item.controller, 2);
    Ok(())
}

fn island(q: &dyn Query, asker: &Asker, id: ObjectId) -> bool {
    q.chars(id).has_subtype(q.words(asker.source).subtype(Subtype::Island))
}

pub static CARD: CardDef = CardDef {
    spell: Some(SpellDef { targets: &[], effect }),
    ways: &[WayDef {
        text: "Return two Islands you control to their owner's hand",
        cost: Cost::return_to_hand(2, island),
        from: ZoneKind::Hand,
        exile_after: false,
        condition: None,
        instant_speed: false,
    }],
    ..oracle::GUSH
};
