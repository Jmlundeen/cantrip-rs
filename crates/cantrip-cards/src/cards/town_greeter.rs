use cantrip_api::*;
use cantrip_mechanics::choose;
use cantrip_mechanics::triggers::this_enters;
use cantrip_oracle as oracle;

/// Mill four cards. You may put a land card from among them into your hand. If you put a Town card into your hand this
/// way, you gain 2 life.
fn mill_and_take(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let player = item.controller;
    let among = g.mill(player, 4);
    let lands: Vec<ObjectId> = among.iter().copied().filter(|&c| g.chars(c).is(CardType::Land)).collect();
    let Some(&card) = choose::cards_from(g, player, item.object, Purpose::PutIntoHand, among, lands, 1, true)?.first() else { return Ok(()) };
    if let Some(&in_hand) = g.return_to_hand(&[card]).first()
        && g.obj(in_hand).zone == Zone::hand(player)
        && g.chars(in_hand).subtypes.has(Subtype::Town)
    {
        g.gain_life(player, 2);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[AbilityDef::Triggered(TriggeredDef {
        text: "When this creature enters, mill four cards. You may put a land card from among them into your hand. If you put a Town card into your hand this way, you gain 2 life.",
        listens: EventMask::ZONE_CHANGE,
        matches: this_enters,
        effect: mill_and_take,
        ..TriggeredDef::BLANK
    })],
    ..oracle::TOWN_GREETER
};
