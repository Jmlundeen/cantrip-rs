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
    spell: spell(&[targets::ARTIFACT_OR_ENCHANTMENT.between(0, 2)], effect),
    ways: &[way(exile_from_hand(1, green_card), ZoneKind::Hand, "Exile a green card from your hand")
        .condition(not_your_turn)],
    ..oracle::FORCE_OF_VIGOR
};
