use cantrip_api::*;
use cantrip_mechanics::{damage_target, targets};
use cantrip_oracle as oracle;

/// Mine Collapse deals 5 damage to target creature or planeswalker.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    damage_target(g, item, 0, 5);
    Ok(())
}

fn mountain(q: &dyn Query, _: &Asker, id: ObjectId) -> bool {
    q.chars(id).has_subtype(Subtype::Mountain)
}

/// "If it's your turn" (CR 118.9): the alternative cost may be paid only on its caster's own turn.
fn your_turn(q: &dyn Query, asker: &Asker) -> bool {
    q.state().turn.active == asker.controller
}

pub static CARD: CardDef = CardDef {
    spell: Some(SpellDef { targets: &[targets::CREATURE_OR_PLANESWALKER], effect }),
    ways: &[WayDef {
        text: "Sacrifice a Mountain",
        cost: Cost::sacrifice(1, mountain),
        from: ZoneKind::Hand,
        exile_after: false,
        condition: Some(your_turn),
        instant_speed: false,
    }],
    ..oracle::MINE_COLLAPSE
};
