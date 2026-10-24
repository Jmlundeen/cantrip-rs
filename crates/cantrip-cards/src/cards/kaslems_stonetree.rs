use cantrip_api::*;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_mechanics::{craft, library};
use cantrip_oracle as oracle;

/// "Look at the top six cards of your library. You may put a land card from among them onto the battlefield tapped. Put
/// the rest on the bottom in a random order."
fn look_for_a_land(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    library::look_at_top_onto_battlefield(g, item.controller, item.object, 6, 1, "land", |q, c| q.chars(c).is(CardType::Land), None, true, library::Rest::BottomRandom)?;
    Ok(())
}

/// "Craft with Cave": a Cave you control or a Cave card from your graveyard (CR 702.167b).
fn cave(q: &dyn Query, _: &Asker, id: ObjectId) -> bool {
    q.chars(id).has_subtype(Subtype::Cave)
}

static CAVE: Materials = craft::materials(1, false, cave, "Cave");

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, look_for_a_land, "When this artifact enters, look at the top six cards of your library. You may put a land card from among them onto the battlefield tapped. Put the rest on the bottom in a random order."),
        craft::ability(mana("{5}{G}"), &CAVE, "Craft with Cave {5}{G}"),
    ],
    ..oracle::KASLEMS_STONETREE
};
