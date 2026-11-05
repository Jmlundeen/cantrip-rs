use cantrip_api::*;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_mechanics::{library, statics, station};
use cantrip_oracle as oracle;

const EIGHT_PLUS: &str = "8+ | Flying, vigilance (3/3)";

/// "When U.S.S. Voyager enters, reveal cards from the top of your library until you reveal two land cards. Put those land
/// cards onto the battlefield tapped and the rest on the bottom of your library in a random order." (CR 701.20, 401.4,
/// 110.2a, 101.4): the lands enter together, and only what was not one of them goes to the bottom.
fn reveal_two_lands(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    let (lands, rest) = library::reveal_until_n(g, you, 2, &|q, card| q.chars(card).is(CardType::Land));
    let entering: Vec<(ObjectId, PlayerId, bool)> = lands.into_iter().map(|land| (land, you, true)).collect();
    g.put_all_onto_battlefield_as(&entering)?;
    library::rest_to_bottom_in_random_order(g, you, &rest);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            THIS_ENTERS,
            reveal_two_lands,
            "When U.S.S. Voyager enters, reveal cards from the top of your library until you reveal two land cards. Put those land cards onto the battlefield tapped and the rest on the bottom of your library in a random order.",
        ),
        station::ability("Station (Tap another creature you control: Put charge counters equal to its power on this Spacecraft. Station only as a sorcery. It's an artifact creature at 8+.)"),
    ],
    statics: &[
        static_ability(Layer::Type, station::charged::<8>, station::is_a_creature, EIGHT_PLUS),
        static_ability(Layer::Ability, station::charged::<8>, statics::have_flying_and_vigilance, EIGHT_PLUS).continues(),
        static_ability(Layer::PtSet, station::charged::<8>, statics::base_pt::<3, 3>, EIGHT_PLUS).continues(),
    ],
    ..oracle::U_S_S_VOYAGER_LONE_WANDERER
};
