use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "Target non-Pilot creature you control": "Pilot" is a word of the spell's text, read through it, so a text-changing
/// effect on the spell changes it (CR 612.2).
fn non_pilot_creature_you_control(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    let pilot = q.words(asker.source).subtype(Subtype::Pilot);
    (targets::CREATURE_YOU_CONTROL.filter)(q, asker, t) && matches!(t, Target::Object(id) if !q.chars(id).has_subtype(pilot))
}

/// Exile target non-Pilot creature you control, then return that card to the battlefield under its owner's
/// control: a new object (CR 400.7). A token exiled ceases to exist, and nothing returns.
fn all_aboard(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    cantrip_mechanics::exile_and_return(g, &[creature])
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

pub static CARD: CardDef = CardDef { enters: Some(cantrip_mechanics::prepare::enters_prepared), prepare: Some(&ALL_ABOARD), ..oracle::SKYCOACH_CONDUCTOR };

pub static EXTRA: &[&CardDef] = &[&ALL_ABOARD];
