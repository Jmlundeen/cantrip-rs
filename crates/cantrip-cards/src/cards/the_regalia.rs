use cantrip_api::*;
use cantrip_mechanics::crew::crew;
use cantrip_mechanics::library;
use cantrip_mechanics::triggers::THIS_ATTACKS;
use cantrip_oracle as oracle;

/// "Whenever The Regalia attacks, reveal cards from the top of your library until you reveal a land card. Put that card onto the
/// battlefield tapped and the rest on the bottom of your library in a random order" (CR 701.20, 401.4): the shared
/// `library::reveal_until_a_land_onto_battlefield_tapped`, which Janice Rand, Essential Aide prints too.
fn reveal_until_a_land(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    library::reveal_until_a_land_onto_battlefield_tapped(g, item.controller)
}

/// Haste is a core keyword.
pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            THIS_ATTACKS,
            reveal_until_a_land,
            "Whenever The Regalia attacks, reveal cards from the top of your library until you reveal a land card. Put that card onto the battlefield tapped and the rest on the bottom of your library in a random order.",
        ),
        crew(1, "Crew 1 (Tap any number of creatures you control with total power 1 or more: This Vehicle becomes an artifact creature until end of turn.)"),
    ],
    ..oracle::THE_REGALIA
};
