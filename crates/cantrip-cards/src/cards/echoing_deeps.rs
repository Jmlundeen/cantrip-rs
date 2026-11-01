use cantrip_api::*;
use cantrip_mechanics::copy;
use cantrip_oracle as oracle;

/// "You may have this land enter tapped as a copy of any land card in a graveyard, except it's a Cave in addition to its
/// other types." A replacement effect on how it enters (CR 614.1c, 707.2): a choice, not a target (CR 608.2d), among the land
/// cards in every graveyard, an opponent's as available as your own. It copies the card's copiable values (CR 707.2) and the
/// exception becomes part of them (CR 707.9b), read through the words of this card's text (CR 612). The copied land's own
/// "enters" replacements and triggers apply as it enters (CR 614.12; the rulings). The card stays in the graveyard. If it copies
/// nothing it enters untapped as itself (the rulings).
fn enter_tapped_as_copy(g: &mut dyn Game, entry: &mut Entering) -> Result<(), Suspend> {
    let cards: Vec<ObjectId> = g
        .state()
        .seats()
        .flat_map(|p| g.state().zone(Zone::graveyard(p)).iter().copied())
        .filter(|&id| g.chars(id).is(CardType::Land))
        .collect();
    copy::enter_tapped_as_copy_of_one_of(g, entry, &cards, |_, copy| copy)?;
    let cave = g.words(entry.object).subtype(Subtype::Cave);
    entry.copy = entry.copy.map(|copy| copy.with_subtype(cave));
    Ok(())
}

pub static CARD: CardDef = CardDef {
    enters_as_copy: Some(enter_tapped_as_copy),
    abilities: &[mana_ability(TAP, add(Mana::C), "{T}: Add {C}.")],
    ..oracle::ECHOING_DEEPS
};
