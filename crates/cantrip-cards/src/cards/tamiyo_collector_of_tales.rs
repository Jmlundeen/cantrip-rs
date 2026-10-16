use cantrip_api::*;
use cantrip_mechanics::{library::object_has_name, targets, top_of_library};
use cantrip_oracle as oracle;

/// "Spells and abilities your opponents control can't cause you to discard cards or sacrifice permanents" (CR 101.2):
/// allowed unless it is her controller who would, and an opponent who causes it.
fn opponents_cant_cause_you(_: &dyn Query, asker: &Asker, causer: PlayerId, player: PlayerId, _sacrifice: bool) -> bool {
    !(player == asker.controller && causer != asker.controller)
}

/// +1: choose a nonland card name, then reveal the top four cards of your library; all cards with that name from among
/// them go into your hand and the rest into your graveyard, in the order their owner chooses (CR 404.3), together.
fn plus_one(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    let ask = RequestKind::ChooseName { source: item.object, looked_at: Vec::new(), suggestion: None, nonland_only: true };
    let Answer::Name(Some(name)) = g.ask(you, ask)? else { return Ok(()) };
    // The four, bottom of them first (as the C# engine lists them).
    let mut top = top_of_library(g, you, 4);
    if top.is_empty() {
        return Ok(());
    }
    top.reverse();
    g.reveal(you, &top);
    let matches: Vec<ObjectId> = top.iter().copied().filter(|&c| object_has_name(g, c, name)).collect();
    let mut rest: Vec<ObjectId> = top.iter().copied().filter(|c| !matches.contains(c)).collect();
    if rest.len() > 1 {
        let ask = RequestKind::DistributeCards {
            source: item.object,
            cards: rest.clone(),
            destinations: vec![Destination { kind: DestinationKind::Graveyard, min: 0, max: u8::MAX, eligible: rest.clone() }],
            may_shuffle: false,
        };
        if let Answer::Placements { placements, .. } = g.ask(you, ask)?
            && let Some(order) = placements.first()
            && order.len() == rest.len()
        {
            rest = order.clone();
        }
    }
    let moves: Vec<(ObjectId, Zone)> =
        matches.iter().map(|&c| (c, Zone::hand(you))).chain(rest.iter().map(|&c| (c, Zone::graveyard(you)))).collect();
    g.move_together(&moves, ZoneReason::Other);
    Ok(())
}

fn card_in_your_graveyard(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    targets::card_in_your_graveyard(q, asker, t, &[])
}

/// -3: return target card from your graveyard to your hand.
fn minus_three(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(card) = targets::object(g, item, 0) {
        g.return_to_hand(&[card]);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    cause_rules: &[opponents_cant_cause_you],
    abilities: &[
        ability(loyalty(1), plus_one, "+1: Choose a nonland card name, then reveal the top four cards of your library. Put all cards with the chosen name from among them into your hand and the rest into your graveyard."),
        ability(loyalty(-3), minus_three, "\u{2212}3: Return target card from your graveyard to your hand.")
            .targets(&[TargetSpec { min: 1, max: 1, filter: card_in_your_graveyard }]),
    ],
    ..oracle::TAMIYO_COLLECTOR_OF_TALES
};
