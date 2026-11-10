use cantrip_api::*;
use cantrip_mechanics::library;
use cantrip_mechanics::triggers::COMMANDER_YOU_CONTROL_ENTERS_OR_ATTACKS;
use cantrip_oracle as oracle;

/// "Whenever a commander you control enters or attacks, put a +1/+1 counter on it, then reveal cards from the top of your library
/// until you reveal a land card. Put that card onto the battlefield tapped and the rest on the bottom of your library in a random
/// order" (CR 603.2, 603.6a, 608.2c, 903.3): the trigger's subject is the commander that entered or attacked - a copy of a commander
/// is not one (CR 707.2) - and the counter is the first instruction, the reveal and put the second, in that order. The reveal half
/// is `library::reveal_until_a_land_onto_battlefield_tapped`, which The Regalia prints too.
fn counter_then_a_land(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Target::Object(commander) = item.subject {
        g.add_counters(commander, CounterKind::PlusOne, 1);
    }
    g.end_sentence();
    library::reveal_until_a_land_onto_battlefield_tapped(g, item.controller)
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        COMMANDER_YOU_CONTROL_ENTERS_OR_ATTACKS,
        counter_then_a_land,
        "Whenever a commander you control enters or attacks, put a +1/+1 counter on it, then reveal cards from the top of your library until you reveal a land card. Put that card onto the battlefield tapped and the rest on the bottom of your library in a random order.",
    )],
    ..oracle::JANICE_RAND_ESSENTIAL_AIDE
};
