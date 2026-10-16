use cantrip_api::*;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_mechanics::{choose, craft, top_of_library};
use cantrip_oracle as oracle;

/// "Look at the top six cards of your library. You may put a land card from among them onto the battlefield tapped. Put
/// the rest on the bottom in a random order." The cards are looked at (a smaller library shows what it has, CR 609.3), up
/// to one land card is chosen, and every other one goes to the bottom in a random order (CR 401.4).
fn look_for_a_land(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    let looked = top_of_library(g, you, 6);
    g.look_at(you, &looked);
    let lands: Vec<ObjectId> = looked.iter().copied().filter(|&c| g.chars(c).is(CardType::Land)).collect();
    let chosen = choose::cards_from_kind(g, you, item.object, Purpose::PutOntoBattlefield, "land", looked.clone(), lands, 1, true)?;
    if let Some(&land) = chosen.first() {
        g.put_onto_battlefield(land, you, true)?;
    }
    let rest: Vec<ObjectId> = looked.into_iter().filter(|c| !chosen.contains(c)).collect();
    g.bottom_in_random_order(you, &rest);
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
