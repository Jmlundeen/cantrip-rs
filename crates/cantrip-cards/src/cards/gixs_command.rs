use cantrip_api::*;
use cantrip_mechanics::choose;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

const ASK: Question = Question::new("PutCounters", "put two +1/+1 counters on a creature?", "It gains lifelink until end of turn", "Choose a creature", "No creature");

fn creatures(g: &dyn Query) -> Vec<ObjectId> {
    g.state().battlefield().iter().copied().filter(|&id| g.chars(id).is(CardType::Creature)).collect()
}

/// Put two +1/+1 counters on up to one creature. It gains lifelink until end of turn. No target: with a creature on
/// the battlefield its controller is asked whether to, and then which (a lone creature is taken without asking).
fn counters(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let candidates = creatures(g);
    if candidates.is_empty() {
        return Ok(());
    }
    if !cantrip_mechanics::effects::ask_may(g, item.controller, item.object, Purpose::Ask(&ASK))? {
        return Ok(());
    }
    let chosen = if candidates.len() == 1 {
        candidates[0]
    } else {
        let ask = RequestKind::ChooseTarget { label: "", slot_player: None, total: None,
            source: item.object,
            purpose: Purpose::Targets,
            candidates: candidates.iter().map(|&id| Target::Object(id)).collect(),
            optional: false,
            least: 1,
            count: 1,
            any_number: false,
        };
        match g.ask(item.controller, ask)? {
            Answer::Targets(t) => t.first().and_then(|t| t.object()).unwrap_or(candidates[0]),
            _ => candidates[0],
        }
    };
    if targets::permanent(g, chosen) {
        g.add_counters(chosen, CounterKind::PlusOne, 2);
        cantrip_mechanics::effects::gain_lifelink_until_end_of_turn(g, item.object, chosen);
    }
    Ok(())
}

/// Destroy each creature with power 2 or less.
fn destroy_small(g: &mut dyn Game, _: &StackItem) -> Result<(), Suspend> {
    let small: Vec<ObjectId> = creatures(g).into_iter().filter(|&id| g.chars(id).power.is_some_and(|p| p <= 2)).collect();
    g.destroy_all(&small);
    Ok(())
}

/// Return up to two creature cards from your graveyard to your hand, together.
fn return_creatures(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let player = item.controller;
    let mut graveyard: Vec<ObjectId> = g.state().zone(Zone::graveyard(player)).to_vec();
    graveyard.reverse();
    let eligible: Vec<ObjectId> = graveyard.iter().copied().filter(|&c| g.chars(c).is(CardType::Creature)).collect();
    let taken = choose::cards_from(g, player, item.object, Purpose::PutIntoHand, graveyard, eligible, 2, true)?;
    g.return_to_hand(&taken);
    Ok(())
}

/// Each opponent sacrifices a creature with the greatest power among creatures they control: chosen by that
/// opponent, in turn order, when several tie; all of them sacrificed together.
fn sacrifice_greatest(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let opponents: Vec<PlayerId> = g.state().apnap().filter(|&p| p != item.controller).collect();
    let mut doomed: Vec<ObjectId> = Vec::new();
    for who in opponents {
        if !cantrip_mechanics::players::reached(g, who) {
            continue;
        }
        let mine: Vec<(ObjectId, i16)> = g
            .permanents_of(who)
            .into_iter()
            .filter(|&id| g.chars(id).is(CardType::Creature))
            .map(|id| (id, g.chars(id).power.unwrap_or(0)))
            .collect();
        let Some(greatest) = mine.iter().map(|&(_, p)| p).max() else { continue };
        let eligible: Vec<ObjectId> = mine.iter().filter(|&&(_, p)| p == greatest).map(|&(id, _)| id).collect();
        if eligible.len() == 1 {
            doomed.push(eligible[0]);
            continue;
        }
        if let Some(card) = choose::card(g, who, item.object, Purpose::Sacrifice, eligible)? {
            doomed.push(card);
        }
    }
    g.sacrifice(&doomed);
    Ok(())
}

static MODAL: ModalDef = modes(&[
    mode(counters, "Put two +1/+1 counters on up to one creature. It gains lifelink until end of turn."),
    mode(destroy_small, "Destroy each creature with power 2 or less."),
    mode(return_creatures, "Return up to two creature cards from your graveyard to your hand."),
    mode(sacrifice_greatest, "Each opponent sacrifices a creature with the greatest power among creatures they control."),
])
    .choose(2, 2);

pub static CARD: CardDef = CardDef { spell: Some(SpellDef::MODAL), modal: Some(&MODAL), ..oracle::GIXS_COMMAND };
