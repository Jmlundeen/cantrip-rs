use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "Your opponents can't cast spells during your turn" (CR 101.2): a cast rule, true for who may cast. The opponents of
/// the permanent's controller may not, while it is that controller's turn.
fn opponents_cant_cast_on_your_turn(q: &dyn Query, asker: &Asker, caster: PlayerId, _from: ZoneKind) -> bool {
    !(q.state().turn.active == asker.controller && caster != asker.controller)
}

// --- The Sensational She-Hulk -----------------------------------------------------------------------------------

/// The one ability of the back face, which a "once each turn" record is kept under.
fn retaliation(q: &dyn Query, source: ObjectId) -> AbilityRef {
    AbilityRef { card: q.chars(source).card, index: 0 }
}

/// "Whenever a creature you control is dealt damage" (CR 120.2): once for each creature dealt some, however many
/// sources dealt it at once (`sums_per_subject`), carrying that much. Not at all once her controller has done the
/// "do this only once each turn" (CR 603.2h).
fn creature_you_control_is_dealt_damage(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::Damage { target: target @ Target::Object(id), amount, .. }
            if *amount > 0
                && q.controller(*id) == q.controller(source)
                && q.chars(*id).is(CardType::Creature)
                && !q.history().done(source, retaliation(q, source)) =>
        {
            Some(Fired { subject: *target, data: [*amount as u32, 0] })
        }
        _ => None,
    }
}

/// "You may have The Sensational She-Hulk deal that much damage to any target. Do this only once each turn." The
/// limit is spent by the doing, not by the triggering: a "no" leaves the turn unspent, and a triggering that waited
/// while another was done finds it spent.
fn retaliate(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if g.history().done(item.object, item.ability) || !g.target_legal(item, 0) {
        return Ok(());
    }
    let amount = item.data[0] as u16;
    if amount == 0 {
        return Ok(());
    }
    if !cantrip_mechanics::effects::ask_may(g, item.controller, item.object, Purpose::Optional)? {
        return Ok(());
    }
    g.note_done(item.object, item.ability);
    g.damage(item.object, item.targets[0], amount);
    Ok(())
}

pub static BACK: CardDef = CardDef {
    abilities: &[triggered(when(EventMask::DAMAGE, creature_you_control_is_dealt_damage), retaliate, "Whenever a creature you control is dealt damage, you may have The Sensational She-Hulk deal that much damage to any target. Do this only once each turn.")
        .targets(&[targets::ANY])
        .sums_per_subject()],
    cast_rules: &[opponents_cant_cast_on_your_turn],
    ..oracle::THE_SENSATIONAL_SHE_HULK
};

pub static CARD: CardDef = CardDef {
    abilities: &[ability(cost("{3}{W}{W}{G}"), cantrip_mechanics::effects::transform_self, "{3}{G}{W}{W}: Transform Jennifer Walters. Activate only as a sorcery.")
        .sorcery_speed()],
    cast_rules: &[opponents_cant_cast_on_your_turn],
    back: Some(&BACK),
    ..oracle::JENNIFER_WALTERS
};
