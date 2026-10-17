use cantrip_api::*;
use cantrip_oracle as oracle;

/// "Whenever another card is put into a graveyard from anywhere" (CR 603.2, 603.6c): any zone change into any graveyard
/// of an object other than this enchantment ("another" needs no test: the enchantment's own move is made as it leaves, when
/// it has no ability left to trigger). A token is not a card (CR 111.1, 108.2), so it never triggers this, though
/// it is in the graveyard for a moment (CR 111.7). The ability is not a leaves-the-battlefield ability, so it does not
/// look back (CR 603.10a): this enchantment must still be on the battlefield after the event, and cards put into a
/// graveyard at the same time as it do not trigger it. The subject is the card in the graveyard, a new object
/// (CR 400.7).
fn another_card_put_into_a_graveyard(q: &dyn Query, _: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::ZoneChange { new, to, .. } if to.is(ZoneKind::Graveyard) && !q.obj(*new).token => {
            Fired::about(Target::Object(*new))
        }
        _ => None,
    }
}

/// "Exile that card." (CR 406.2): the card as it is in the graveyard; if it has left it since, it is a new object that
/// this ability does not know (CR 400.7), and `exile_all` leaves it alone.
fn exile_that_card(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Target::Object(card) = item.subject {
        g.exile_all(&[card]);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        when(EventMask::ZONE_CHANGE, another_card_put_into_a_graveyard),
        exile_that_card,
        "Whenever another card is put into a graveyard from anywhere, exile that card.",
    )],
    ..oracle::PLANAR_VOID
};
