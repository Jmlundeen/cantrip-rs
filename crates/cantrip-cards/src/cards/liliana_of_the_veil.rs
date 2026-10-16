use cantrip_api::*;
use cantrip_oracle as oracle;
use cantrip_mechanics::{choose, choose_from_hand, targets};

/// +1: each player discards a card, chosen in turn order from the active player (CR 101.4), discarded together.
fn plus_one(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let mut chosen = Vec::new();
    for player in g.state().apnap().collect::<Vec<_>>() {
        chosen.extend(choose_from_hand(g, player, item.object, 1)?);
    }
    g.discard_together(&chosen);
    Ok(())
}

/// -2: target player sacrifices a creature, their choice when they have more than one.
fn minus_two(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(victim) = item.targets[0].player() else { return Ok(()) };
    cantrip_mechanics::sacrifice_chosen(g, victim, item.object, 1, |c| c.is(CardType::Creature))
}

/// -6: separate all permanents target player controls into two piles (the ability's controller chooses the first;
/// the rest are the second, CR 700.3), and that player sacrifices the pile of their choice.
fn minus_six(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(victim) = item.targets[0].player() else { return Ok(()) };
    let permanents = g.permanents_of(victim);
    let first: Vec<ObjectId> = if permanents.is_empty() {
        Vec::new()
    } else {
        let candidates: Vec<Target> = permanents.iter().map(|&id| Target::Object(id)).collect();
        let count = candidates.len() as u8;
        let ask = RequestKind::ChooseTarget { label: "", slot_player: None,
            source: item.object,
            purpose: Purpose::SeparatePiles,
            candidates,
            optional: true,
            least: 0,
            count,
            any_number: true,
        };
        match g.ask(item.controller, ask)? {
            Answer::Targets(targets) => targets.iter().filter_map(|t| t.object()).collect(),
            _ => Vec::new(),
        }
    };
    let second: Vec<ObjectId> = permanents.iter().copied().filter(|id| !first.contains(id)).collect();
    let piles = vec![Label::Pile { first: true, empty: first.is_empty() }, Label::Pile { first: false, empty: second.is_empty() }];
    let pile = if choose::one_of(g, victim, item.object, piles)? == 1 { second } else { first };
    g.sacrifice(&pile);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        ability(loyalty(1), plus_one, "+1: Each player discards a card."),
        ability(loyalty(-2), minus_two, "\u{2212}2: Target player sacrifices a creature.")
            .targets(&[targets::PLAYER]),
        ability(loyalty(-6), minus_six, "\u{2212}6: Separate all permanents target player controls into two piles. That player sacrifices all permanents in the pile of their choice.")
            .targets(&[targets::PLAYER]),
    ],
    ..oracle::LILIANA_OF_THE_VEIL
};
