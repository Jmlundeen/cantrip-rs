use cantrip_api::*;
use cantrip_mechanics::replacements::exile_instead_apply;
use cantrip_mechanics::{choose, choose_from_hand, targets, tokens};
use cantrip_oracle as oracle;

/// "If a creature an opponent controls would die" (CR 614.1a, 700.4): a creature controlled by an opponent of this
/// permanent's controller, moving from the battlefield to a graveyard. A token is a creature too.
fn opponents_creature_would_die(q: &dyn Query, asker: &Asker, proposal: &Proposal) -> bool {
    matches!(proposal, Proposal::ZoneChange(z)
        if z.from.is(ZoneKind::Battlefield)
            && z.to.is(ZoneKind::Graveyard)
            && q.chars(z.object).is(CardType::Creature)
            && q.state().opponents(asker.controller).any(|o| o == q.controller(z.object)))
}

/// +2: Up to one target creature gets -4/-1 until your next turn (CR 611.2b).
fn plus_two(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(creature) = targets::object(g, item, 0)
        && g.is_on_battlefield(creature)
    {
        g.add_effect(item.object, creature, EffectKind::Pump(-4, -1), Duration::UntilNextTurnOf(item.controller));
    }
    Ok(())
}

/// −2: Each player sacrifices a creature of their choice. If you sacrificed a creature this way, create a 4/4 green Beast
/// creature token with trample. Each player chooses in turn order (CR 101.4) and the creatures are sacrificed together
/// (CR 701.21a); a player with a single creature has no choice.
fn minus_two(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    let mut doomed: Vec<ObjectId> = Vec::new();
    let mut yours: Option<ObjectId> = None;
    for who in g.state().apnap().collect::<Vec<_>>() {
        let creatures: Vec<ObjectId> = g.permanents_of(who).into_iter().filter(|&id| g.chars(id).is(CardType::Creature)).collect();
        let chosen = match creatures.len() {
            0 => None,
            1 => Some(creatures[0]),
            _ => choose::card(g, who, item.object, Purpose::Sacrifice, creatures)?,
        };
        if let Some(creature) = chosen {
            doomed.push(creature);
            if who == you {
                yours = Some(creature);
            }
        }
    }
    g.sacrifice(&doomed);
    if let Some(creature) = yours
        && g.obj(creature).became.is_some()
    {
        tokens::create(g, you, &tokens::BEAST_4_4_TRAMPLE, 1)?;
    }
    Ok(())
}

/// −3: Each opponent discards two cards. For each opponent who didn't discard two nonland cards this way, you draw a
/// card. Each opponent chooses in turn order and the cards are discarded together (CR 101.4, 701.9); an opponent with
/// fewer cards, or whom this ability can't make discard (CR 101.2), is one who didn't discard two nonland cards.
fn minus_three(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    let opponents: Vec<PlayerId> = g.state().apnap().filter(|&p| p != you).collect();
    let mut chosen: Vec<ObjectId> = Vec::new();
    for &who in &opponents {
        chosen.extend(choose_from_hand(g, who, item.object, 2)?);
    }
    let discarded = g.discard_together(&chosen);
    let short = opponents
        .iter()
        .filter(|&&who| {
            discarded.iter().filter(|&&card| g.obj(card).owner == who && !g.chars(card).is(CardType::Land)).count() < 2
        })
        .count();
    for _ in 0..short {
        g.draw(you, 1);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    replacements: &[replacement(ReplacementKind::ZoneChange, opponents_creature_would_die, exile_instead_apply, "exiles a creature an opponent controls instead of letting it die")],
    abilities: &[
        ability(loyalty(2), plus_two, "+2: Up to one target creature gets -4/-1 until your next turn.").targets(&[targets::CREATURE.optional()]),
        ability(loyalty(-2), minus_two, "\u{2212}2: Each player sacrifices a creature of their choice. If you sacrificed a creature this way, create a 4/4 green Beast creature token with trample."),
        ability(loyalty(-3), minus_three, "\u{2212}3: Each opponent discards two cards. For each opponent who didn't discard two nonland cards this way, you draw a card."),
    ],
    ..oracle::GARRUK_VEILED_BUTCHER
};

pub static EXTRA: &[&CardDef] = &[&tokens::BEAST_4_4_TRAMPLE];
