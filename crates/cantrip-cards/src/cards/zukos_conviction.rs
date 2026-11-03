use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "Return target creature card from your graveyard to your hand. If this spell was kicked, instead put that card onto the
/// battlefield tapped." (CR 608.2b, 702.33): a target gone from the graveyard by resolution is simply not moved. Kicked, the
/// card enters under your control (CR 110.2) instead of going to your hand, never both.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let cards = targets::legal_objects(g, item);
    if item.kicked > 0 {
        let rows: Vec<(ObjectId, PlayerId, bool)> = cards.iter().map(|&card| (card, item.controller, true)).collect();
        g.put_all_onto_battlefield_as(&rows)?;
    } else {
        g.return_to_hand(&cards);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    kicker: kicker(mana("{4}")),
    spell: spell(&[targets::CREATURE_CARD_IN_YOUR_GRAVEYARD], effect),
    ..oracle::ZUKOS_CONVICTION
};
