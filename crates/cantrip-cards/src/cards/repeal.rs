use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "Target nonland permanent with mana value X" (CR 107.3a, 202.3): X is announced before the target is chosen
/// (`late_targets`) and judged again as the spell resolves. Before it is announced, with the card still in a hand, every
/// nonland permanent is allowed, since some X would allow it.
fn nonland_permanent_with_mana_value_x(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    let Target::Object(id) = t else { return false };
    if !(targets::NONLAND_PERMANENT.filter)(q, asker, t) {
        return false;
    }
    let spell = q.obj(asker.source);
    !spell.zone.is(ZoneKind::Stack) || q.mana_value(id) == spell.x as u32
}

/// Return target nonland permanent with mana value X to its owner's hand. Draw a card.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(permanent) = targets::object(g, item, 0) {
        g.return_to_hand(&[permanent]);
    }
    g.draw(item.controller, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[target(nonland_permanent_with_mana_value_x)], effect),
    late_targets: true,
    ..oracle::REPEAL
};
