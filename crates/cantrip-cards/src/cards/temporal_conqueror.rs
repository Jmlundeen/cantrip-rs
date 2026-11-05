use cantrip_api::*;
use cantrip_mechanics::select::{self, exactly};
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_mechanics::{choose, tokens};
use cantrip_oracle as oracle;

/// "When this creature enters, mill three cards. Then return a land card from your graveyard to your hand. If you can't,
/// create a Treasure token." (CR 701.17, 111.10a, 608.2c): the mill happens first, so a land card it put into the
/// graveyard may be the one returned; a graveyard with no land card in it at that point can't return one, and then the
/// Treasure is made. Nothing is chosen when there is only nothing to return.
fn enter(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    g.mill(you, 3);
    g.end_sentence();
    let lands: Vec<ObjectId> = g
        .state()
        .zone(Zone::graveyard(you))
        .iter()
        .copied()
        .filter(|&card| g.chars(card).is(CardType::Land))
        .collect();
    if lands.is_empty() {
        tokens::create_treasure(g, you)?;
    } else {
        let chosen = choose::select(g, you, item.object, Purpose::PutIntoHand, lands, select::land_cards(exactly(1)))?;
        g.return_to_hand(&chosen);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        THIS_ENTERS,
        enter,
        "When this creature enters, mill three cards. Then return a land card from your graveyard to your hand. If you can't, create a Treasure token.",
    )],
    ..oracle::TEMPORAL_CONQUEROR
};

pub static EXTRA: &[&CardDef] = &[&tokens::TREASURE];
