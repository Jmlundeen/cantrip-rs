use cantrip_api::*;
use cantrip_mechanics::choose;
use cantrip_oracle as oracle;

/// "Mill four cards. Then put an instant, sorcery, or Faerie card from among the milled cards into your hand." The card is
/// not optional: one is chosen whenever any milled card qualifies (CR 701.17c). An Adventurer card in a graveyard has only
/// its normal characteristics (CR 715.4), so it is an instant or sorcery card never, a Faerie card when its creature is one.
fn mill_and_take(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let player = item.controller;
    let among = g.mill(player, 4);
    let faerie = g.words(item.object).subtype(Subtype::Faerie);
    let eligible: Vec<ObjectId> = among
        .iter()
        .copied()
        .filter(|&c| {
            let chars = g.chars(c);
            chars.is(CardType::Instant) || chars.is(CardType::Sorcery) || chars.has_subtype(faerie)
        })
        .collect();
    let chosen = choose::cards_from(g, player, item.object, Purpose::PutIntoHand, among, eligible, 1, false)?;
    g.return_to_hand(&chosen);
    Ok(())
}

/// The Adventure (CR 715): Free the Fae.
pub static FREE_THE_FAE: CardDef = CardDef {
    spell: spell(&[], mill_and_take),
    ..oracle::FREE_THE_FAE
};

pub static CARD: CardDef = CardDef {
    adventure: Some(&FREE_THE_FAE),
    ..oracle::PICKLOCK_PRANKSTER
};

pub static EXTRA: &[&CardDef] = &[&FREE_THE_FAE];
