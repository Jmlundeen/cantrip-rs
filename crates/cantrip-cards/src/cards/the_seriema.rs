use cantrip_api::*;
use cantrip_mechanics::{library, select, statics, station, triggers::THIS_ENTERS};
use cantrip_oracle as oracle;

const SEVEN_PLUS: &str = "7+ | Flying";

/// "Search your library for a legendary creature card, reveal it, put it into your hand, then shuffle" (CR 701.23b, 701.20,
/// 701.24): the player may fail to find, and the search itself shuffles.
fn fetch_a_legendary_creature(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    library::find_to_hand(g, item.controller, item.object, select::LEGENDARY_CREATURE_CARD, true)?;
    Ok(())
}

/// "Other tapped legendary creatures you control": a legendary creature you control other than this one, tapped now (CR 613.1f).
fn other_tapped_legendary_creatures_you_control(q: &dyn LayerQuery, source: ObjectId, affected: ObjectId, so_far: &Chars) -> bool {
    statics::other_creatures_you_control(q, source, affected, so_far) && so_far.supertypes.legendary && q.state().obj(affected).tapped
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            THIS_ENTERS,
            fetch_a_legendary_creature,
            "When The Seriema enters, search your library for a legendary creature card, reveal it, put it into your hand, then shuffle.",
        ),
        station::ability("Station (Tap another creature you control: Put charge counters equal to its power on this Spacecraft. Station only as a sorcery. It's an artifact creature at 7+.)"),
    ],
    statics: &[
        static_ability(Layer::Type, station::charged::<7>, station::is_a_creature, SEVEN_PLUS),
        static_ability(Layer::Ability, station::charged::<7>, statics::have_flying, SEVEN_PLUS).continues(),
        static_ability(Layer::PtSet, station::charged::<7>, statics::base_pt::<5, 5>, SEVEN_PLUS).continues(),
        static_ability(
            Layer::Ability,
            other_tapped_legendary_creatures_you_control,
            statics::have_indestructible,
            "Other tapped legendary creatures you control have indestructible.",
        ),
    ],
    ..oracle::THE_SERIEMA
};
