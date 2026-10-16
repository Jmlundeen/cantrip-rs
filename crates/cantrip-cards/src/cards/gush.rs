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
    spell: spell(&[], effect),
    ways: &[way(Cost::return_to_hand(2, island), ZoneKind::Hand, "Return two Islands you control to their owner's hand")],
    ..oracle::GUSH
};
