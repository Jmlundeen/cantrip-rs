use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// Who controls a spell on the stack (CR 112.2), if it is one.
fn spell_controller(q: &dyn Query, id: ObjectId) -> Option<PlayerId> {
    q.state().stack.iter().find(|i| i.kind == StackKind::Spell && i.object == id).map(|i| i.controller)
}

/// Exchange control of target noncreature spell and target creature. Then the spell's controller may choose new targets
/// for it. The exchange is all or nothing (CR 701.12a): with either target illegal, or both controlled by one player
/// (CR 701.12b), nothing changes hands. The spell changes hands on the stack (CR 112.2); the creature stays an effect
/// of control with no end (CR 613.1b). Whoever controls the spell afterwards, the creature's old controller after an
/// exchange, its caster when none happened, may then choose new targets for it (CR 115.7d).
fn exchange(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let (Target::Object(spell), Target::Object(creature)) = (item.target(0), item.target(1)) else { return Ok(()) };
    if g.target_legal(item, 0) && g.target_legal(item, 1) {
        let spell_by = spell_controller(g, spell);
        let creature_by = g.controller(creature);
        if let Some(spell_by) = spell_by
            && spell_by != creature_by
        {
            g.change_controller(spell, creature_by);
            g.add_effect(item.object, creature, EffectKind::Control(spell_by), Duration::Indefinite);
        }
    }
    // Still on the stack, even when the exchange did not happen: its controller may retarget it.
    if let Some(controller) = spell_controller(g, spell) {
        g.choose_new_targets(spell, controller)?;
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    keywords: Keywords::of(&[Keyword::SplitSecond]),
    spell: spell(&[targets::NONCREATURE_SPELL, targets::CREATURE], exchange),
    ..oracle::SUDDEN_SUBSTITUTION
};
