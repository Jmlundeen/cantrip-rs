use cantrip_api::*;
use cantrip_oracle as oracle;

/// "Return all creature cards with mana value 2 or less from your graveyard to the battlefield." Every such card enters at
/// the same moment, under your control (they are put onto the battlefield by an effect of yours, CR 110.2a; 603.6a: each enters-the-battlefield ability sees all of them
/// enter). Mana value is read from the graveyard (CR 202.3: an {X} there is 0, so Mockingbird's is 1).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let back: Vec<(ObjectId, PlayerId)> = g
        .state()
        .zone(Zone::graveyard(item.controller))
        .iter()
        .copied()
        .filter(|&card| g.chars(card).is(CardType::Creature) && g.mana_value(card) <= 2)
        .map(|card| (card, item.controller))
        .collect();
    if !back.is_empty() {
        g.put_all_onto_battlefield(&back)?;
    }
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: spell(&[], effect), ..oracle::RAISE_THE_PAST };
