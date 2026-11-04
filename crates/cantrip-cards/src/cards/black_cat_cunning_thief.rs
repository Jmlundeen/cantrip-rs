use cantrip_api::*;
use cantrip_mechanics::{choose, library, select, targets, top_of_library, triggers};
use cantrip_oracle as oracle;

/// "When Black Cat enters, look at the top nine cards of target opponent's library, exile two of them face down, then put the
/// rest on the bottom of their library in a random order. You may play the exiled cards for as long as they remain exiled. Mana
/// of any type can be spent to cast spells this way." (CR 401.4, 406.3, 609.4b, 118.14, 305.1.) A library of fewer than nine
/// shows what it has, and of fewer than two gives up what it has (CR 609.3). The controller looks at the nine, chooses the two,
/// and goes on knowing them; whatever a replacement kept out of exile is not played. The permission is the controller's, to
/// play the cards (a land among them as a land play) while each stays in exile. With the target no longer legal nothing
/// happens (CR 608.2b).
fn look_exile_two(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(opponent) = targets::player(g, item, 0) else { return Ok(()) };
    let you = item.controller;
    let looked = top_of_library(g, opponent, 9);
    g.look_at(you, &looked);
    let chosen = choose::select(g, you, item.object, Purpose::Exile, looked.clone(), select::cards(select::as_many_as_can(2)))?;
    let grant = PlayGrant { lands: true, any_type: true, ..PlayGrant::CAST };
    library::exile_face_down_may_look_and_play(g, &chosen, you, grant);
    g.end_sentence();
    library::rest_to_bottom_in_random_order(g, opponent, &looked);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        triggers::THIS_ENTERS,
        look_exile_two,
        "When Black Cat enters, look at the top nine cards of target opponent's library, exile two of them face down, then put the rest on the bottom of their library in a random order. You may play the exiled cards for as long as they remain exiled. Mana of any type can be spent to cast spells this way.",
    )
    .targets(&[targets::OPPONENT])],
    ..oracle::BLACK_CAT_CUNNING_THIEF
};
