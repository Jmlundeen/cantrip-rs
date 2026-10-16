use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

fn non_pilot_creature_you_control(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    (targets::CREATURE_YOU_CONTROL.filter)(q, asker, t) && matches!(t, Target::Object(id) if !q.chars(id).has_subtype(Subtype::Pilot))
}

/// Exile target non-Pilot creature you control, then return that card to the battlefield under its owner's
/// control: a new object (CR 400.7). A token exiled ceases to exist, and nothing returns.
fn all_aboard(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    let token = g.obj(creature).token;
    let owner = g.obj(creature).owner;
    let exiled = g.exile(creature);
    if token || !g.obj(exiled).zone.is(ZoneKind::Exile) {
        return Ok(());
    }
    g.put_onto_battlefield(exiled, owner, false)?;
    Ok(())
}

/// Skycoach Conductor's prepare spell (CR 722): not a card, only ever a copy.
pub static ALL_ABOARD: CardDef = CardDef {
    name: "All Aboard",
    cost: mana("{U}"),
    types: TypeSet::of(&[CardType::Instant]),
    spell: spell(&[target(non_pilot_creature_you_control)], all_aboard),
    text: "Exile target non-Pilot creature you control, then return that card to the battlefield under its owner's control.",
    ..CardDef::BLANK
};

fn enters_prepared(_: &mut dyn Game, entering: &mut Entering) -> Result<(), Suspend> {
    entering.prepared = true;
    Ok(())
}

pub static CARD: CardDef = CardDef { enters: Some(enters_prepared), prepare: Some(&ALL_ABOARD), ..oracle::SKYCOACH_CONDUCTOR };

pub static EXTRA: &[&CardDef] = &[&ALL_ABOARD];
