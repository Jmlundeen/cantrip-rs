use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

fn green_card(q: &dyn Query, asker: &Asker, id: ObjectId) -> bool {
    q.chars(id).colors.has(q.words(asker.source).color(Color::Green))
}

/// "If it's not your turn" (CR 118.9): the alternative cost is offered only during another player's turn.
fn not_your_turn(q: &dyn Query, asker: &Asker) -> bool {
    q.state().turn.active != asker.controller
}

/// Destroy each target that is still legal, together.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.destroy_all(&targets::legal_objects(g, item));
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: Some(SpellDef { targets: &[TargetSpec { min: 0, max: 2, ..targets::ARTIFACT_OR_ENCHANTMENT }], effect }),
    ways: &[WayDef {
        text: "Exile a green card from your hand",
        cost: Cost { exile_from_hand: Some((1, green_card)), ..Cost::NONE },
        from: ZoneKind::Hand,
        exile_after: false,
        condition: Some(not_your_turn),
        instant_speed: false,
    }],
    ..oracle::FORCE_OF_VIGOR
};
