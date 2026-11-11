use cantrip_api::*;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_mechanics::{effects, library, select, station, targets};
use cantrip_oracle as oracle;

const THREE_PLUS: &str = "3+ | {T}: Target creature can't be blocked this turn.";

/// "When this Spacecraft enters, mill five cards. You may put up to two artifact cards from among them into your hand."
/// (CR 701.17a, 608.2d): the five milled cards are the ones offered, artifact cards only, up to two of them.
fn mill_five_then_up_to_two_artifacts(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    library::mill_then_up_to_two_to_hand(g, item, 5, select::artifact_cards(select::up_to(2)))
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, mill_five_then_up_to_two_artifacts, "When this Spacecraft enters, mill five cards. You may put up to two artifact cards from among them into your hand."),
        station::ability("Station (Tap another creature you control: Put charge counters equal to its power on this Spacecraft. Station only as a sorcery.)"),
    ],
    // The 3+ striation (CR 721.2a): while it has three or more charge counters it has the {T} ability in its `delayed` list.
    statics: &[static_ability(Layer::Ability, station::charged::<3>, station::has::<0>, THREE_PLUS)],
    delayed: &[ability(TAP, effects::target_creature_cant_be_blocked_this_turn, "{T}: Target creature can't be blocked this turn.")
        .targets(&[targets::CREATURE])],
    ..oracle::TRANSWARP_HUB
};
