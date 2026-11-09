use cantrip_api::*;
use cantrip_mechanics::{choose, select};
use cantrip_oracle as oracle;

/// "{T}, Sacrifice this land: Put your commander into your hand from the command zone." (CR 903.3, 903.9b, 608.2d):
/// only a commander card in the command zone that this player owns; with two designated commanders its controller chooses
/// one (CR 903.3), and with none the ability does nothing. Moving it to hand is a normal zone change, so the next
/// cast from hand pays no commander tax (CR 903.8).
fn return_commander(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    let commanders: Vec<ObjectId> = g
        .state()
        .zone(Zone::COMMAND)
        .iter()
        .copied()
        .filter(|&id| g.obj(id).commander && g.obj(id).owner == you)
        .collect();
    let chosen = choose::select(g, you, item.object, Purpose::ReturnToHand, commanders, select::cards(select::exactly(1)))?;
    if let Some(&chosen) = chosen.first() {
        g.return_to_hand(&[chosen]);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add(Mana::C), "{T}: Add {C}."),
        ability(
            TAP.and(SACRIFICE_THIS),
            return_commander,
            "{T}, Sacrifice this land: Put your commander into your hand from the command zone.",
        ),
    ],
    ..oracle::COMMAND_BEACON
};
