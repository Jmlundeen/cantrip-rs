use cantrip_api::*;
use cantrip_mechanics::triggers::{you_attack_with_a_creature_of_subtype, LANDFALL};
use cantrip_mechanics::{choose, effects, lands, select, top_of_library};
use cantrip_oracle as oracle;

/// "Look at that many cards from the top of your library. You may put one of them into your hand. Then put any number of land
/// cards from among them onto the battlefield tapped and the rest into your graveyard" (CR 701.20a, 608.2c): the number is
/// that of the Birds that attacked together. A library with fewer cards shows what it has (CR 609.3). The land cards left after
/// the one taken into the hand enter together (CR 110.2a); the others still in the library go to the graveyard together.
fn look_take_one_lands_onto_battlefield_rest_to_graveyard(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let player = item.controller;
    let count = g.event_objects(item).len();
    let looked = top_of_library(g, player, count);
    if looked.is_empty() {
        return Ok(());
    }
    g.look_at(player, &looked);
    let taken = choose::select(g, player, item.object, Purpose::PutIntoHand, looked.clone(), select::cards(select::up_to(1)))?;
    if !taken.is_empty() {
        g.return_to_hand(&taken);
    }
    let left: Vec<ObjectId> = looked.into_iter().filter(|c| !taken.contains(c)).collect();
    let lands = choose::select(g, player, item.object, Purpose::PutOntoBattlefield, left.clone(), select::land_cards(select::ANY_NUMBER))?;
    if !lands.is_empty() {
        let entering: Vec<(ObjectId, PlayerId, bool)> = lands.iter().map(|&c| (c, player, true)).collect();
        g.put_all_onto_battlefield_as(&entering)?;
    }
    let rest: Vec<ObjectId> = left.into_iter().filter(|c| !lands.contains(c)).collect();
    g.put_into_graveyard(&rest);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            when_asked(EventMask::ATTACKS, you_attack_with_a_creature_of_subtype::<{ lands::sub(Subtype::Bird) }>),
            look_take_one_lands_onto_battlefield_rest_to_graveyard,
            "Whenever one or more Birds you control attack, look at that many cards from the top of your library. You may put one of them into your hand. Then put any number of land cards from among them onto the battlefield tapped and the rest into your graveyard.",
        )
        .once_per_batch(),
        triggered(LANDFALL, effects::this_gets::<1, 0>, "Landfall \u{2014} Whenever a land you control enters, this creature gets +1/+0 until end of turn."),
    ],
    ..oracle::CHOCO_SEEKER_OF_PARADISE
};
