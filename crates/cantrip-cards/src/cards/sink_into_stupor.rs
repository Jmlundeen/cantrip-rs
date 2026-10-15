use cantrip_api::*;
use cantrip_mechanics::mana;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// Who controls a spell on the stack, or a permanent on the battlefield, if it is either (CR 109.4, 112.2).
fn controller_of(q: &dyn Query, id: ObjectId) -> Option<PlayerId> {
    if let Some(item) = q.state().stack.iter().find(|i| i.kind == StackKind::Spell && i.object == id) {
        return Some(item.controller);
    }
    (targets::permanent(q, id)).then(|| q.controller(id))
}

/// "Target spell or nonland permanent an opponent controls" (CR 115.1, 109.5): judged again as it resolves.
fn spell_or_nonland_permanent_of_an_opponent(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    let Target::Object(id) = t else { return false };
    let Some(controller) = controller_of(q, id) else { return false };
    controller != asker.controller && (q.state().stack.iter().any(|i| i.kind == StackKind::Spell && i.object == id) || !q.chars(id).is(CardType::Land))
}

/// Return target spell or nonland permanent an opponent controls to its owner's hand. A spell is not countered: it
/// just leaves the stack (CR 400.3).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(id) = targets::object(g, item, 0) else { return Ok(()) };
    cantrip_mechanics::effects::return_spell_or_permanent_to_hand(g, id);
    Ok(())
}

pub static BACK: CardDef = CardDef {
    abilities: &[mana::tap_for(mana::add_u, Mana::U, "{T}: Add {U}.")],
    enters: Some(mana::enters_tapped_unless_pay_three_life),
    ..oracle::SOPORIFIC_SPRINGS
};

pub static CARD: CardDef = CardDef {
    spell: Some(SpellDef {
        targets: &[TargetSpec { min: 1, max: 1, filter: spell_or_nonland_permanent_of_an_opponent }],
        effect,
    }),
    back: Some(&BACK),
    ..oracle::SINK_INTO_STUPOR
};
