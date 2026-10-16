use cantrip_api::*;
use cantrip_mechanics::choose;
use cantrip_mechanics::library::ask_library_order;
use cantrip_mechanics::mana;
use cantrip_oracle as oracle;

/// Put any number of cards from your hand on the bottom of your library, then draw that many cards plus one. Any
/// number is chosen (none is an answer), their owner orders them when there is more than one (CR 401.4), they move
/// together, and "that many" is how many arrived in a library: one instruction to draw that many plus one (CR 121.2a).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let who = item.controller;
    let hand = g.state().zone(Zone::hand(who)).to_vec();
    let mut bottomed = 0;
    if !hand.is_empty() {
        let count = hand.len().min(u8::MAX as usize) as u8;
        let cards = choose::cards(g, who, item.object, Purpose::PutOnBottom, hand, count, true)?;
        let order = ask_library_order(g, who, item.object, &cards, DestinationKind::LibraryBottom)?;
        bottomed = g.put_into_library(&order, LibraryPlace::Bottom).len() as u32;
    }
    g.draw(who, bottomed + 1);
    Ok(())
}

pub static BACK: CardDef = CardDef {
    abilities: &[mana_ability(TAP, add(Mana::R), "{T}: Add {R}.")],
    enters: Some(mana::enters_tapped),
    ..oracle::VALAKUT_STONEFORGE
};

pub static CARD: CardDef = CardDef {
    spell: spell(&[], effect),
    back: Some(&BACK),
    ..oracle::VALAKUT_AWAKENING
};
