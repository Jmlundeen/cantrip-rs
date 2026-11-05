use cantrip_api::*;
use cantrip_mechanics::{effects, targets, triggers, triggers::THIS_ENTERS};
use cantrip_oracle as oracle;

/// The Omen (CR 720): Absorb Essence, "Target creature gets +2/+2 and gains lifelink and hexproof until end of turn", one
/// instruction on the spell's first target (CR 611.2a, 608.2b). Cast from the hand as an Omen instant for {1}{W}; as it resolves it
/// is shuffled into its owner's library (CR 720.3d, the core's).
fn pump_and_gain_lifelink_and_hexproof(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    instruction(g, |g| {
        cantrip_mechanics::pump(g, item.object, creature, 2, 2);
        effects::gain_until_end_of_turn(g, item.object, creature, Keyword::Lifelink);
        effects::gain_until_end_of_turn(g, item.object, creature, Keyword::Hexproof);
    });
    Ok(())
}

pub static ABSORB_ESSENCE: CardDef = CardDef {
    spell: spell(&[targets::CREATURE], pump_and_gain_lifelink_and_hexproof),
    ..oracle::ABSORB_ESSENCE
};

/// "Remove all counters from up to one target creature": every kind it has, all of each (CR 122.1), from a creature still a legal
/// target as the ability resolves (CR 608.2b). Vampire Hexmage prints the same for any permanent.
fn remove_all_counters(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    effects::remove_all_counters(g, creature);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggers::ward_life::<2>("Ward\u{2014}Pay 2 life."),
        triggered(THIS_ENTERS, remove_all_counters, "When this creature enters, remove all counters from up to one target creature.")
            .targets(&[targets::CREATURE.between(0, 1)]),
    ],
    omen: Some(&ABSORB_ESSENCE),
    ..oracle::PURGING_STORMBROOD
};

pub static EXTRA: &[&CardDef] = &[&ABSORB_ESSENCE];
