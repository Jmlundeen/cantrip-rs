use cantrip_api::*;
use cantrip_mechanics::{targets, counter_unless_pays};
use cantrip_oracle as oracle;

/// Counter target spell unless its controller pays {1} (CR 118.12).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    counter_unless_pays(g, item, 0, mana("{1}"))?;
    Ok(())
}

fn island(q: &dyn Query, asker: &Asker, id: ObjectId) -> bool {
    q.chars(id).has_subtype(q.words(asker.source).subtype(Subtype::Island))
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::SPELL], effect),
    ways: &[way(Cost::return_to_hand(1, island), ZoneKind::Hand, "Return an Island you control to its owner's hand")],
    ..oracle::DAZE
};
