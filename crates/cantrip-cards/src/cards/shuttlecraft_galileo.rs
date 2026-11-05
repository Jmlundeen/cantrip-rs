use cantrip_api::*;
use cantrip_mechanics::{station, statics, tokens, triggers};
use cantrip_oracle as oracle;

const SIX_PLUS: &str = "6+ | Flying";

/// "Landfall — Whenever a land you control enters, create a 1/1 red Officer creature token" (CR 603.6a): one token for each
/// land its controller's lands bring in, however they entered (CR 305.1).
fn make_officer(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &tokens::OFFICER, 1)?;
    Ok(())
}

/// "Whenever Shuttlecraft Galileo attacks, up to seven target attacking creatures gain flying until end of turn" (CR 601.2c,
/// 611.2a): the targets are chosen as the trigger goes on the stack, and each still attacking and still a creature as it
/// resolves gains flying (CR 608.2b, 506.4).
fn grant_flying(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let creatures = cantrip_mechanics::targets::legal_objects(g, item);
    instruction(g, |g| {
        for creature in creatures {
            cantrip_mechanics::effects::gain_until_end_of_turn(g, item.object, creature, Keyword::Flying);
        }
    });
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        station::ability("Station (Tap another creature you control: Put charge counters equal to its power on this Spacecraft. Station only as a sorcery. It's an artifact creature at 6+.)"),
        triggered(triggers::LANDFALL, make_officer, "Landfall — Whenever a land you control enters, create a 1/1 red Officer creature token."),
        triggered(
            triggers::THIS_ATTACKS,
            grant_flying,
            "Whenever Shuttlecraft Galileo attacks, up to seven target attacking creatures gain flying until end of turn.",
        )
        .targets(&[cantrip_mechanics::targets::ATTACKING_CREATURE.between(0, 7)]),
    ],
    statics: &[
        static_ability(Layer::Type, station::charged::<6>, station::is_a_creature, SIX_PLUS),
        static_ability(Layer::Ability, station::charged::<6>, statics::have_flying, SIX_PLUS).continues(),
        static_ability(Layer::PtSet, station::charged::<6>, statics::base_pt::<3, 3>, SIX_PLUS).continues(),
    ],
    ..oracle::SHUTTLECRAFT_GALILEO
};

pub static EXTRA: &[&CardDef] = &[&tokens::OFFICER];
