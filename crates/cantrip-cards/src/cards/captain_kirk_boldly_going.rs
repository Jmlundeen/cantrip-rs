use cantrip_api::select::part;
use cantrip_api::*;
use cantrip_mechanics::library;
use cantrip_mechanics::select::{up_to, Part, Selection};
use cantrip_mechanics::statics::plus_pt;
use cantrip_mechanics::triggers::THIS_ENTERS_OR_ATTACKS;
use cantrip_mechanics::lands;
use cantrip_oracle as oracle;

const ASK: Question = Question::new(
    "SearchLibrary",
    "search your library and graveyard for a basic land card or a card named Starship Enterprise?",
    "Reveal it and put it into your hand, then shuffle. If you don't, you search only your graveyard and must take a card that fits",
    "Search both",
    "Graveyard only",
);

/// "Spacecraft creatures you control get +3/+3." (CR 613.4c, layer 7c, 612.2): every creature of the controller's with the
/// Spacecraft subtype, judged on what the earlier layers made each object - a Spacecraft is an artifact creature only while
/// it has the charge counters its station ability names, so one below that is no creature and gets nothing - and by control,
/// not ownership. The source itself is a Human Officer, so it is never among them.
fn your_spacecraft_creatures(q: &dyn LayerQuery, source: ObjectId, affected: ObjectId, so_far: &Chars) -> bool {
    so_far.is(CardType::Creature)
        && q.controller(affected) == q.controller(source)
        && so_far.has_subtype(q.words(source).subtype(Subtype::Spacecraft))
}

/// "A basic land card or a card named Starship Enterprise" (CR 701.23, 201.2a): a basic land card, or a card one of whose
/// names is that printed name, as it is in the zone searched.
fn basic_land_or_starship_enterprise(q: &dyn Query, choice: &Choice, _: u32) -> Verdict {
    Verdict::each(choice, |card| {
        lands::is_basic_land_card(q, card) || library::object_names(q, card).contains(&"Starship Enterprise")
    })
}

const BASIC_LAND_OR_STARSHIP_ENTERPRISE: Part = part(&(basic_land_or_starship_enterprise as SelectFn), 0, "");

/// One such card, or none: a search for a quality may find nothing (CR 701.23b).
const FETCHED: Selection =
    Selection::of(up_to(1), &[BASIC_LAND_OR_STARSHIP_ENTERPRISE]).called("basic land card or a card named Starship Enterprise");

/// "Whenever Captain Kirk enters or attacks, search your library and/or graveyard for a basic land card or a card named
/// Starship Enterprise, reveal it, and put it into your hand. If you search your library this way, shuffle." (CR 701.23,
/// 701.20a, 701.24): the one card is chosen among both zones, or left unfound in the library (CR 701.23b); a
/// graveyard-only search is no library search, and the graveyard, being public, must give up a card that fits.
fn enters_or_attacks(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    library::search_zone_and_or_library_to_hand(g, item.controller, item.object, &[Zone::graveyard(item.controller)], &ASK, FETCHED)
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        THIS_ENTERS_OR_ATTACKS,
        enters_or_attacks,
        "Whenever Captain Kirk enters or attacks, search your library and/or graveyard for a basic land card or a card named Starship Enterprise, reveal it, and put it into your hand. If you search your library this way, shuffle.",
    )],
    statics: &[static_ability(
        Layer::PtModify,
        your_spacecraft_creatures,
        plus_pt::<3, 3>,
        "Spacecraft creatures you control get +3/+3.",
    )],
    ..oracle::CAPTAIN_KIRK_BOLDLY_GOING
};
