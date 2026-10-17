use cantrip_api::*;
use cantrip_mechanics::library::{search_to_graveyard, Find};
use cantrip_mechanics::{choose, spree};
use cantrip_oracle as oracle;

/// The total mana value of the creature cards returned by the second mode.
const BUDGET: u32 = 4;

/// What the creature chosen next may be, by the mana value left of the four (the question says it).
const KINDS: [&str; 5] = [
    "creature card with mana value 0",
    "creature card with mana value 1 or less (4 or less in total)",
    "creature card with mana value 2 or less (4 or less in total)",
    "creature card with mana value 3 or less (4 or less in total)",
    "creature card with mana value 4 or less",
];

/// "Search your library for a card, put it into your graveyard, then shuffle." (CR 701.23.)
fn search(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    search_to_graveyard(g, item.controller, item.object, Find::ANY, |_, _| true)?;
    Ok(())
}

/// "Return up to two creature cards with total mana value 4 or less from your graveyard to the battlefield." The cards are
/// chosen as the mode resolves, the whole graveyard shown (a card the search just put there is among them: the modes are
/// followed in printed order, CR 700.2): one at a time, the second within what is left of the four, which allows exactly
/// the pairs whose total is 4 or less (CR 601.2c does not apply: not targets). They enter together (CR 110.2a).
fn return_creatures(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    let mut chosen: Vec<ObjectId> = Vec::new();
    let mut left = BUDGET;
    for _ in 0..2 {
        let graveyard = g.state().zone(Zone::graveyard(you)).to_vec();
        let eligible: Vec<ObjectId> = graveyard
            .iter()
            .copied()
            .filter(|&c| !chosen.contains(&c) && g.chars(c).is(CardType::Creature) && g.mana_value(c) <= left)
            .collect();
        let Some(card) = choose::cards_from_kind(g, you, item.object, Purpose::PutOntoBattlefield, KINDS[left as usize], graveyard, eligible, 1, true)?
            .first()
            .copied()
        else {
            break;
        };
        left -= g.mana_value(card);
        chosen.push(card);
    }
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
