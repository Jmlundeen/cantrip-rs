use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// A nonland, nontoken permanent the caster controls (CR 115.1).
fn nonland_nontoken_permanent_you_control(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    (targets::NONLAND_PERMANENT.filter)(q, asker, t)
        && matches!(t, Target::Object(id) if q.controller(id) == asker.controller && !q.obj(id).token)
}

/// "Return any number of target nonland, nontoken permanents you control to their owners' hands. Draw a card for each
/// card returned to your hand this way": the targets still legal go back together, each to its owner's hand, and the
/// draw counts only those that reached the caster's own hand.
fn return_and_draw(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    let returning: Vec<ObjectId> =
        (0..item.target_count as usize).filter(|&slot| g.target_legal(item, slot)).filter_map(|slot| item.targets[slot].object()).collect();
    let returned = g.return_to_hand(&returning).into_iter().filter(|&new| g.obj(new).zone == Zone::hand(you)).count();
    if returned > 0 {
        g.draw(you, returned as u32);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: Some(SpellDef {
        targets: &[TargetSpec { min: 0, max: 16, filter: nonland_nontoken_permanent_you_control, label: "" }],
        effect: return_and_draw,
    }),
    ..oracle::PARADOXICAL_OUTCOME
};
