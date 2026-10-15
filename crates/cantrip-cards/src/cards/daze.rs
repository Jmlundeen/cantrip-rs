use cantrip_api::*;
use cantrip_mechanics::{targets, counter_unless_pays};
use cantrip_oracle as oracle;

/// Counter target spell unless its controller pays {1} (CR 118.12).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    counter_unless_pays(g, item, 0, mana(1, 0, 0, 0, 0, 0))?;
    Ok(())
}

fn island(q: &dyn Query, _: &Asker, id: ObjectId) -> bool {
    q.chars(id).has_subtype(Subtype::Island)
}

pub static CARD: CardDef = CardDef {
    spell: Some(SpellDef { targets: &[targets::SPELL], effect }),
    ways: &[WayDef {
        text: "Return an Island you control to its owner's hand",
        cost: Cost::return_to_hand(1, island),
        from: ZoneKind::Hand,
        exile_after: false,
        condition: None,
        instant_speed: false,
    }],
    ..oracle::DAZE
};
