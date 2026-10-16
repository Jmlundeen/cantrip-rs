use cantrip_api::*;
use cantrip_mechanics::library::{self, Find};
use cantrip_mechanics::statics::{plus_pt, your_creatures};
use cantrip_mechanics::tokens::{self, GNOME};
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_mechanics::craft;
use cantrip_oracle as oracle;

/// "Search your library for a basic Plains card, reveal it, put it into your hand, then shuffle. You gain 2 life." The
/// Plains is a basic land type read through the Bricks' words (CR 612); a search for a quality may find nothing (CR
/// 701.23b), and the life is gained either way.
fn fetch_plains(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let plains = g.words(item.object).subtype(Subtype::Plains);
    library::search_to_hand(g, item.controller, item.object, Find::ONE.revealed(), move |q, card| {
        let chars = q.chars(card);
        chars.supertypes.basic && chars.has_subtype(plains)
    })?;
    g.gain_life(item.controller, 2);
    Ok(())
}

/// "When this artifact enters, create two 1/1 colorless Gnome artifact creature tokens."
fn two_gnomes(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &GNOME, 2)?;
    Ok(())
}

pub static BACK: CardDef = CardDef {
    abilities: &[triggered(THIS_ENTERS, two_gnomes, "When this artifact enters, create two 1/1 colorless Gnome artifact creature tokens.")],
    statics: &[static_ability(Layer::PtModify, your_creatures, plus_pt::<1, 1>, "Creatures you control get +1/+1.")],
    ..oracle::COSMIUM_KILN
};

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, fetch_plains, "When this artifact enters, search your library for a basic Plains card, reveal it, put it into your hand, then shuffle. You gain 2 life."),
        craft::ability(mana("{5}{W}{W}"), &craft::ARTIFACT, "Craft with artifact {5}{W}{W}"),
    ],
    back: Some(&BACK),
    ..oracle::CLAY_FIRED_BRICKS
};

pub static EXTRA: &[&CardDef] = &[&GNOME];
