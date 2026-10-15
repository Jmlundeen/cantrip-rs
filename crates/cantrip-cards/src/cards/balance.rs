use cantrip_api::*;
use cantrip_mechanics::choose;
use cantrip_oracle as oracle;

/// Each player, in turn order, chooses which of their cards to lose so they keep as many as the player with the
/// fewest has (asked only when they lose some but not all), then all of them go together. Lands and creatures are
/// sacrificed, cards in hand discarded.
fn equalize(g: &mut dyn Game, source: ObjectId, from_hand: bool, kind: Option<CardType>) -> Result<(), Suspend> {
    let players: Vec<PlayerId> = g.state().apnap().collect();
    let theirs = |g: &dyn Query, who: PlayerId| -> Vec<ObjectId> {
        if from_hand {
            g.state().zone(Zone::hand(who)).to_vec()
        } else {
            g.permanents_of(who).into_iter().filter(|&id| kind.is_none_or(|k| g.chars(id).is(k))).collect()
        }
    };
    let Some(fewest) = players.iter().map(|&who| theirs(g, who).len()).min() else { return Ok(()) };
    let mut going: Vec<ObjectId> = Vec::new();
    for &who in &players {
        let mine = theirs(g, who);
        if mine.len() <= fewest {
            continue;
        }
        let count = mine.len() - fewest;
        if count == mine.len() {
            going.extend(mine);
            continue;
        }
        let purpose = if from_hand { Purpose::Discard } else { Purpose::Sacrifice };
        let cards = choose::cards(g, who, source, purpose, mine.clone(), count as u8, false)?;
        going.extend(mine.iter().copied().filter(|c| cards.contains(c)));
    }
    if from_hand {
        g.discard_together(&going);
    } else {
        g.sacrifice(&going);
    }
    Ok(())
}

fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    equalize(g, item.object, false, Some(CardType::Land))?;
    equalize(g, item.object, true, None)?;
    equalize(g, item.object, false, Some(CardType::Creature))
}

pub static CARD: CardDef = CardDef { spell: Some(SpellDef { targets: &[], effect }), ..oracle::BALANCE };
