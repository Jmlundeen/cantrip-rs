use cantrip_api::*;
use cantrip_mechanics::escape;
use cantrip_oracle as oracle;

/// "Each nonland card in your graveyard has escape. The escape cost is equal to the card's mana cost plus exile three
/// other cards from your graveyard" (CR 702.138a, 113.6, 604): for the cards in its controller's graveyard that have a
/// mana cost, cast by that controller.
fn escape_cost(q: &dyn Query, asker: &Asker, card: ObjectId, caster: PlayerId) -> Option<Cost> {
    if caster != asker.controller || q.obj(card).owner != asker.controller {
        return None;
    }
    let def = q.card(q.obj(card).card);
    if def.is(CardType::Land) || !def.has_cost {
        return None;
    }
    Some(escape::cost(def.cost, 3))
}

/// "At the beginning of the end step, sacrifice this enchantment": every end step, not only its controller's.
fn end_step(_: &dyn Query, _: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::StepBegan { step: Step::End } => Fired::PLAIN,
        _ => None,
    }
}

fn sacrifice_this(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if g.is_on_battlefield(item.object) && g.controller(item.object) == item.controller {
        g.sacrifice(&[item.object]);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    granted_ways: &[GrantedWay { text: "Escape", cost: escape_cost, ..GrantedWay::CAST }],
    abilities: &[AbilityDef::Triggered(TriggeredDef {
        text: "At the beginning of the end step, sacrifice this enchantment.",
        listens: EventMask::STEP_BEGAN,
        matches: end_step,
        effect: sacrifice_this,
        ..TriggeredDef::BLANK
    })],
    ..oracle::UNDERWORLD_BREACH
};
