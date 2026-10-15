use cantrip_api::*;
use cantrip_mechanics::{choose, goyf, triggers, effects};
use cantrip_oracle as oracle;

/// You may mill that many cards; if you do, you may put a creature card from among them into your hand (CR 701.17c).
fn mill_and_take(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if !effects::ask_may(g, item.controller, item.object, Purpose::Optional)? {
        return Ok(());
    }
    let player = item.controller;
    let among = g.mill(player, item.data[0]);
    let eligible: Vec<ObjectId> = among.iter().copied().filter(|&c| g.chars(c).is(CardType::Creature)).collect();
    let chosen = choose::cards_from(g, player, item.object, Purpose::PutIntoHand, among, eligible, 1, true)?;
    g.return_to_hand(&chosen);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[AbilityDef::Triggered(TriggeredDef {
        text: "Whenever this creature deals combat damage to a player, you may mill that many cards. If you do, you may put a creature card from among them into your hand.",
        listens: EventMask::DAMAGE,
        matches: triggers::deals_combat_damage_to_player_subject,
        effect: mill_and_take,
        ..TriggeredDef::BLANK
    })],
    statics: &[goyf::SIZE],
    ..oracle::BARROWGOYF
};
