use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_oracle as oracle;

/// "At the beginning of each player's upkeep" (CR 503.1): the ability is about that player, "the first player".
fn each_upkeep(q: &dyn Query, _: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::StepBegan { step: Step::Upkeep } => Fired::about(Target::Player(q.state().turn.active)),
        _ => None,
    }
}

fn creatures(q: &dyn Query, player: PlayerId) -> usize {
    q.permanents_of(player).into_iter().filter(|&id| q.chars(id).is(CardType::Creature)).count()
}

/// "Target player who controls more creatures than they do and is their opponent": measured against the player whose
/// upkeep it is, not against the enchantment's controller.
fn player_ahead_on_creatures(q: &dyn Query, _: &Asker, t: Target) -> bool {
    let Target::Player(other) = t else { return false };
    let them = q.state().turn.active;
    other != them && q.state().opponents(them).any(|p| p == other) && creatures(q, other) > creatures(q, them)
}

/// The first player may reveal cards from the top of their library until they reveal a creature card. If they do, they put
/// that card onto the battlefield and all other cards revealed this way into their graveyard. The whole reveal is worked out
/// before anything moves; a library with no creature card is revealed to the last card and every one of them is buried.
fn reveal(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Target::Player(who) = item.subject else { return Ok(()) };
    if !effects::ask_may(g, who, item.object, Purpose::Optional)? {
        return Ok(());
    }
    let mut revealed: Vec<ObjectId> = Vec::new();
    let mut creature = None;
    for card in g.state().zone(Zone::library(who)).iter().copied() {
        revealed.push(card);
        if g.chars(card).is(CardType::Creature) {
            creature = Some(card);
            break;
        }
    }
    if revealed.is_empty() {
        return Ok(());
    }
    g.reveal(who, &revealed);
    let buried: Vec<(ObjectId, Zone)> =
        revealed.iter().copied().filter(|&c| Some(c) != creature).map(|c| (c, Zone::graveyard(who))).collect();
    if !buried.is_empty() {
        g.move_together(&buried, ZoneReason::Other);
    }
    if let Some(card) = creature {
        g.put_onto_battlefield(card, who, false)?;
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(when(EventMask::STEP_BEGAN, each_upkeep), reveal, "At the beginning of each player's upkeep, that player chooses target player who controls more creatures than they do and is their opponent. The first player may reveal cards from the top of their library until they reveal a creature card. If the first player does, that player puts that card onto the battlefield and all other cards revealed this way into their graveyard.")
        .targets(&[TargetSpec { min: 1, max: 1, filter: player_ahead_on_creatures, label: "" }])
        .subject_chooses_targets()],
    ..oracle::OATH_OF_DRUIDS
};
