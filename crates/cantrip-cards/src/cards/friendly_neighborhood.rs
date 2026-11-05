use cantrip_api::*;
use cantrip_mechanics::statics::grant_delayed_ability;
use cantrip_mechanics::tokens::{self, HUMAN_CITIZEN};
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_mechanics::{attach::attached_object, creatures_you_control, pump, targets};
use cantrip_oracle as oracle;

/// "Create three 1/1 green and white Human Citizen creature tokens": together (CR 101.4), under the Aura's controller.
fn three_citizens(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &HUMAN_CITIZEN, 3)?;
    Ok(())
}

/// "Target creature gets +1/+1 until end of turn for each creature you control" (CR 611.2a): counted as the ability resolves, by the
/// player who controls the ability - the land's controller, since the ability is the land's own (CR 113.8) - the target itself counting if it is
/// one of theirs. Nothing if the target is no longer legal (CR 608.2b).
fn pump_for_each_creature_you_control(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    let count = creatures_you_control(g, item.controller) as i32;
    pump(g, item.object, creature, count, count);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    enchant: Some(targets::LAND),
    abilities: &[triggered(
        THIS_ENTERS,
        three_citizens,
        "When this Aura enters, create three 1/1 green and white Human Citizen creature tokens.",
    )],
    delayed: &[ability(
        cost("{1}").and(TAP),
        pump_for_each_creature_you_control,
        "{1}, {T}: Target creature gets +1/+1 until end of turn for each creature you control. Activate only as a sorcery.",
    )
    .targets(&[targets::CREATURE])
    .sorcery_speed()],
    statics: &[static_ability(
        Layer::Ability,
        attached_object,
        grant_delayed_ability,
        "Enchanted land has \"{1}, {T}: Target creature gets +1/+1 until end of turn for each creature you control. Activate only as a sorcery.\"",
    )],
    ..oracle::FRIENDLY_NEIGHBORHOOD
};

pub static EXTRA: &[&CardDef] = &[&HUMAN_CITIZEN];
