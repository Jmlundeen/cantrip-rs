use cantrip_api::*;
use cantrip_mechanics::{effects, haste_until_end_of_turn, mana, station, targets, tokens};
use cantrip_oracle as oracle;

/// "Create a 2/2 colorless Robot artifact creature token, then creatures you control get +1/+0 and gain haste until end of
/// turn" (CR 111.1, 611.2c): the creatures are those its controller controls as the ability resolves, the new Robot among
/// them, and none that enters later.
fn robot_then_pump_and_haste(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &tokens::ROBOT_2_2, 1)?;
    for creature in effects::pump_creatures_you_control(g, item, 1, 0) {
        haste_until_end_of_turn(g, item.object, creature);
    }
    Ok(())
}

pub static EXTRA: &[&CardDef] = &[&tokens::ROBOT_2_2];

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add(Mana::R), "{T}: Add {R}."),
        station::ability("Station (Tap another creature you control: Put charge counters equal to its power on this Planet. Station only as a sorcery.)"),
    ],
    // "This land enters tapped" (CR 614.1c).
    enters: Some(mana::enters_tapped),
    // The 12+ striation (CR 721.2a): an activated ability Kavaron has while it has twelve or more charge counters. Its
    // controller may sacrifice Kavaron itself as the land (it is tapped as a cost first, CR 602.2b).
    delayed: &[ability(
        cost("{1}{R}").and(TAP).and(sacrifice(1, targets::filters::land)),
        robot_then_pump_and_haste,
        "{1}{R}, {T}, Sacrifice a land: Create a 2/2 colorless Robot artifact creature token, then creatures you control get +1/+0 and gain haste until end of turn.",
    )],
    statics: &[static_ability(
        Layer::Ability,
        station::charged::<12>,
        station::has::<0>,
        "12+ | {1}{R}, {T}, Sacrifice a land: Create a 2/2 colorless Robot artifact creature token, then creatures you control get +1/+0 and gain haste until end of turn.",
    )],
    ..oracle::KAVARON_MEMORIAL_WORLD
};
