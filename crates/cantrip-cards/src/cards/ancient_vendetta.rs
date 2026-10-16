use cantrip_api::*;
use cantrip_mechanics::library::{self, object_has_name, Find};
use cantrip_mechanics::{choose, targets};
use cantrip_oracle as oracle;

/// Up to this many cards in all, whichever zones they are found in.
const MOST: u8 = 4;

/// "Choose a card name. Search target opponent's graveyard, hand, and library for up to four cards with that name and exile
/// them. Then that player shuffles." The name is chosen as the spell resolves, not as it is cast (CR 608.2c), and may be any
/// card name, a basic land's included (CR 201.4a). The caster searches the three zones in the order the text names them,
/// choosing among the cards with the name in each, in all up to four (a player need not take them all); everything found
/// is exiled at once and the owner shuffles (CR 701.23, 406.2). A spell whose only target is illegal does not resolve
/// (CR 608.2b), so no name is chosen then.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    let Some(owner) = item.targets[0].player() else { return Ok(()) };
    let name = g.ask(you, RequestKind::ChooseName { source: item.object, looked_at: Vec::new(), suggestion: None, kind: NameKind::Any })?;
    // A card has "that name" when it is the card name chosen; with no name chosen nothing has it.
    let named = |q: &dyn Query, c: ObjectId| matches!(name, Answer::Name(Some(wanted)) if object_has_name(q, c, wanted));
    let graveyard: Vec<ObjectId> = g.state().zone(Zone::graveyard(owner)).iter().copied().filter(|&c| named(g, c)).collect();
    let mut found = choose::cards(g, you, item.object, Purpose::Exile, graveyard, MOST, true)?;
    let hand: Vec<ObjectId> = g.state().zone(Zone::hand(owner)).iter().copied().filter(|&c| named(g, c)).collect();
    let remaining = MOST - found.len() as u8;
    found.extend(choose::cards(g, you, item.object, Purpose::Exile, hand, remaining, true)?);
    let remaining = MOST - found.len() as u8;
    library::search_to_exile(g, you, owner, item.object, Find::ONE.up_to(remaining), &found, named)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::OPPONENT], effect),
    ..oracle::ANCIENT_VENDETTA
};
