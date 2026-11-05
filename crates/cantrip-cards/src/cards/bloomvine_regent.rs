use cantrip_api::*;
use cantrip_mechanics::{effects, library, select, triggers};
use cantrip_oracle as oracle;

/// "Up to two basic Forest cards" (CR 205.4a, 205.3i, 701.23b): basic, with the subtype Forest read through the words of the
/// text of the card that asks (CR 612.2); a nonbasic Forest is not one.
const BASIC_FOREST_CARDS: select::Selection =
    select::Selection::of(select::up_to(2), &[select::basic_land_of_type(Subtype::Forest)]).called("basic Forest");

/// "Search your library for up to two basic Forest cards, reveal them, put one onto the battlefield tapped and the other into
/// your hand, then shuffle." (CR 701.23, 701.20, 701.24): the player finds none, one or two. A lone card found goes onto the
/// battlefield tapped; of two, the player says which one does, and the other goes into the hand, revealed. The library is
/// shuffled whether or not anything was found.
fn claim_territory(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    library::search_two_one_onto_battlefield_tapped_other_into_hand(g, item, BASIC_FOREST_CARDS)
}

/// The Omen (CR 720): cast from the hand as an Omen sorcery for {2}{G}; as it resolves it is shuffled into its owner's library
/// (CR 720.3d, the core's).
pub static CLAIM_TERRITORY: CardDef = CardDef { spell: spell(&[], claim_territory), ..oracle::CLAIM_TERRITORY };

pub static CARD: CardDef = CardDef {
    // "This creature or another Dragon you control" (CR 603.6a): a Dragon under your control as it enters, this one included.
    abilities: &[triggered(
        triggers::DRAGON_YOU_CONTROL_ENTERS,
        effects::gain_three_life,
        "Whenever this creature or another Dragon you control enters, you gain 3 life.",
    )],
    omen: Some(&CLAIM_TERRITORY),
    ..oracle::BLOOMVINE_REGENT
};

pub static EXTRA: &[&CardDef] = &[&CLAIM_TERRITORY];
