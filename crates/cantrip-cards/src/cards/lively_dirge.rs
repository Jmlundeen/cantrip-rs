use cantrip_api::*;
use cantrip_mechanics::library::{search_to_graveyard, Find};
use cantrip_mechanics::{choose, spree};
use cantrip_oracle as oracle;

/// "Search your library for a card, put it into your graveyard, then shuffle." (CR 701.23.)
fn search(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    search_to_graveyard(g, item.controller, item.object, Find::ANY, |_, _| true)?;
    Ok(())
}

/// "Return up to two creature cards with total mana value 4 or less from your graveyard to the battlefield." The cards are
/// chosen together as the mode resolves, the whole graveyard shown (a card the search just put there is among them: the modes
/// are followed in printed order, CR 700.2), their mana values adding up to 4 or less (CR 202.3), which the question itself
/// enforces; they are not targets (CR 601.2c does not apply). They enter together (CR 110.2a).
fn return_creatures(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    let graveyard = g.state().zone(Zone::graveyard(you)).to_vec();
    let creatures: Vec<ObjectId> = graveyard.iter().copied().filter(|&c| g.chars(c).is(CardType::Creature)).collect();
    let chosen = choose::cards_within_total(g, you, item.object, Purpose::PutOntoBattlefield, "creature", graveyard, creatures, 2, 4)?;
    let entering: Vec<(ObjectId, PlayerId)> = chosen.into_iter().map(|c| (c, you)).collect();
    g.put_all_onto_battlefield(&entering)?;
    Ok(())
}

const MODES: &[ModeDef] = &[
    mode(search, "+ {1} \u{2014} Search your library for a card, put it into your graveyard, then shuffle.").cost(mana("{1}")),
    mode(return_creatures, "+ {2} \u{2014} Return up to two creature cards with total mana value 4 or less from your graveyard to the battlefield.").cost(mana("{2}")),
];

static MODAL: ModalDef = spree::modal(MODES);

pub static CARD: CardDef = CardDef { spell: Some(SpellDef::MODAL), modal: Some(&MODAL), ..oracle::LIVELY_DIRGE };
