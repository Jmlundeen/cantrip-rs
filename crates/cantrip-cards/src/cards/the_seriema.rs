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

/// "Other tapped legendary creatures you control": a legendary creature its holder's controller controls other than the
/// holder, tapped now (CR 613.1f).
fn other_tapped_legendary_creatures_you_control(q: &dyn LayerQuery, source: ObjectId, affected: ObjectId, so_far: &Chars) -> bool {
    statics::other_creatures_you_control(q, source, affected, so_far) && so_far.supertypes.legendary && q.state().obj(affected).tapped
}

/// The 7+ striation's layer 6 part: flying and the static ability "Other tapped legendary creatures you control have
/// indestructible", together, so that both continue the one effect that began in layer 4 (CR 613.6, 721.2a-b) and The Seriema
/// has them under an earlier "loses all abilities"; a later one takes them both (CR 613.7).
fn flying_and_the_indestructible_grant(q: &dyn LayerQuery, source: ObjectId, chars: &mut Chars) {
    statics::have_flying(q, source, chars);
    station::has::<0>(q, source, chars);
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
        static_ability(Layer::Ability, station::charged::<7>, flying_and_the_indestructible_grant, SEVEN_PLUS).continues(),
        static_ability(Layer::PtSet, station::charged::<7>, statics::base_pt::<5, 5>, SEVEN_PLUS).continues(),
    ],
    // The 7+ striation's static ability (CR 721.2a), which The Seriema has while it has seven or more charge counters.
    delayed: &[granted_static_ability(
        Layer::Ability,
        other_tapped_legendary_creatures_you_control,
        statics::have_indestructible,
        "Other tapped legendary creatures you control have indestructible.",
    )],
    ..oracle::THE_SERIEMA
};
