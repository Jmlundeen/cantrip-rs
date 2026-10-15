use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

fn blue_card(q: &dyn Query, _: &Asker, id: ObjectId) -> bool {
    q.chars(id).colors.has(Color::Blue)
}

/// "If it's not your turn" (CR 118.9): the alternative cost is offered only during another player's turn.
fn not_your_turn(q: &dyn Query, asker: &Asker) -> bool {
    q.state().turn.active != asker.controller
}

/// Counter target noncreature spell. If it is countered this way, exile it instead of putting it into its owner's
/// graveyard.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(spell) = targets::object(g, item, 0) {
        g.counter_to(spell, CounterTo::Exile);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: Some(SpellDef { targets: &[targets::NONCREATURE_SPELL], effect }),
    ways: &[WayDef {
        text: "Exile a blue card from your hand",
        cost: Cost { exile_from_hand: Some((1, blue_card)), ..Cost::NONE },
        from: ZoneKind::Hand,
        exile_after: false,
        condition: Some(not_your_turn),
        instant_speed: false,
    }],
    ..oracle::FORCE_OF_NEGATION
};
