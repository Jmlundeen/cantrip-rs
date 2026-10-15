use cantrip_api::*;
use cantrip_oracle as oracle;

/// "Target spell that targets you or a permanent you control" (CR 115.1): a spell on the stack, one of whose targets is
/// the player this ability's controller is, or a permanent on the battlefield that player controls. Asked as the spell
/// is cast and again as it resolves (CR 608.2b), so a spell whose targets changed, or a permanent that changed hands
/// or left (a new object, CR 400.7), stops being a legal target.
fn targets_you_or_yours(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    let Target::Object(spell) = t else { return false };
    let Some(item) = q.state().stack.iter().find(|i| i.kind == StackKind::Spell && i.object == spell) else { return false };
    item.targets().iter().any(|&aimed| match aimed {
        Target::Player(p) => p == asker.controller,
        Target::Object(id) => cantrip_mechanics::targets::permanent(q, id) && q.controller(id) == asker.controller,
        _ => false,
    })
}

/// Counter target spell that targets you or a permanent you control. Draw a card. A spell whose only target has become
/// illegal does nothing at all (CR 608.2b).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if cantrip_mechanics::counter_target(g, item, 0).is_some() {
        g.draw(item.controller, 1);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: Some(SpellDef { targets: &[TargetSpec { min: 1, max: 1, filter: targets_you_or_yours }], effect }),
    ..oracle::HINDERING_LIGHT
};
