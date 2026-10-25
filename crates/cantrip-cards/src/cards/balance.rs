use cantrip_api::*;
use cantrip_mechanics::choose;
use cantrip_oracle as oracle;

/// Each player, in turn order, chooses which of their cards to lose so they keep as many as the player with the
/// fewest has (asked only when they lose some but not all), then all of them go together. Cards in hand are discarded:
/// the player chooses the ones to discard. Lands and creatures are sacrificed as the text words it - the player "chooses
/// a number of lands they control equal to [the fewest], then sacrifices the rest" - so they choose the ones they keep,
/// and of the rest one that can't be sacrificed stays (CR 101.2) without making them sacrifice a kept one instead.
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
    let mut going: Vec<(PlayerId, ObjectId)> = Vec::new();
    for &who in &players {
        if !cantrip_mechanics::players::reached(g, who) {
            continue;
        }
        let mine = theirs(g, who);
        if mine.len() <= fewest {
            continue;
        }
        let count = mine.len() - fewest;
        if count == mine.len() {
            going.extend(mine.into_iter().map(|c| (who, c)));
            continue;
        }
        if from_hand {
            let cards = choose::cards(g, who, source, Purpose::Discard, mine.clone(), u32::try_from(count).unwrap_or(u32::MAX), false)?;
            going.extend(mine.iter().copied().filter(|c| cards.contains(c)).map(|c| (who, c)));
        } else {
            let kept = choose::cards(g, who, source, Purpose::Keep, mine.clone(), u32::try_from(fewest).unwrap_or(u32::MAX), false)?;
            going.extend(mine.iter().copied().filter(|c| !kept.contains(c)).map(|c| (who, c)));
        }
    }
    if from_hand {
        g.discard_together(&going.iter().map(|&(_, c)| c).collect::<Vec<_>>());
    } else {
        g.sacrifice_each(&going);
    }
    Ok(())
}

fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    equalize(g, item.object, false, Some(CardType::Land))?;
    equalize(g, item.object, true, None)?;
    equalize(g, item.object, false, Some(CardType::Creature))
}

pub static CARD: CardDef = CardDef { spell: spell(&[], effect), ..oracle::BALANCE };
