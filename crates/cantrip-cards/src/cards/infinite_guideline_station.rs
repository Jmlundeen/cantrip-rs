use cantrip_api::*;
use cantrip_mechanics::tokens::{self, ROBOT_2_2};
use cantrip_mechanics::triggers::{THIS_ATTACKS, THIS_ENTERS};
use cantrip_mechanics::{statics, station};
use cantrip_oracle as oracle;

const TWELVE_PLUS: &str = "12+ | Flying";

/// The permanents `player` controls that are two or more colors (CR 105.2b), as the ability resolves: the Station itself, all five
/// colors from its mana cost, is one.
fn multicolored_permanents_you_control(q: &dyn Query, player: PlayerId) -> u32 {
    q.permanents_of(player).into_iter().filter(|&id| q.chars(id).colors.0.count_ones() >= 2).count() as u32
}

/// "When Infinite Guideline Station enters, create a tapped 2/2 colorless Robot artifact creature token for each multicolored
/// permanent you control" (CR 701.7): counted once as the ability resolves, the tokens made together (CR 101.4).
fn create_robots(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let count = multicolored_permanents_you_control(g, item.controller);
    tokens::create_by_rule(g, item.controller, &ROBOT_2_2, count, true)?;
    Ok(())
}

/// "Draw a card for each multicolored permanent you control" (CR 121.2): counted as the ability resolves, only the attacking
/// player's permanents.
fn draw_per_multicolored_permanent(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let count = multicolored_permanents_you_control(g, item.controller);
    g.draw(item.controller, count);
    Ok(())
}

/// Flying is granted by the 12+ striation, not printed on the card (CR 721.2a).
pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            THIS_ENTERS,
            create_robots,
            "When Infinite Guideline Station enters, create a tapped 2/2 colorless Robot artifact creature token for each multicolored permanent you control.",
        ),
        station::ability(
            "Station (Tap another creature you control: Put charge counters equal to its power on this Spacecraft. Station only as a sorcery. It's an artifact creature at 12+.)",
        ),
        triggered(
            THIS_ATTACKS,
            draw_per_multicolored_permanent,
            "Whenever Infinite Guideline Station attacks, draw a card for each multicolored permanent you control.",
        ),
    ],
    statics: &[
        static_ability(Layer::Type, station::charged::<12>, station::is_a_creature, TWELVE_PLUS),
        static_ability(Layer::Ability, statics::itself, statics::have_flying, TWELVE_PLUS).continues(),
        static_ability(Layer::PtSet, statics::itself, statics::base_pt::<7, 15>, TWELVE_PLUS).continues(),
    ],
    ..oracle::INFINITE_GUIDELINE_STATION
};

pub static EXTRA: &[&CardDef] = &[&ROBOT_2_2];
