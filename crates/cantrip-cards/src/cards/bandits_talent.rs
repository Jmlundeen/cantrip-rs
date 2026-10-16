use cantrip_api::*;
use cantrip_mechanics::triggers::{self, THIS_ENTERS};
use cantrip_mechanics::{choose_discard_two_unless_discarding, class};
use cantrip_oracle as oracle;

/// "When this Class enters, each opponent discards two cards unless they discard a nonland card." Each opponent, in
/// turn order (CR 101.4), chooses between one nonland card and two cards of any kind (CR 608.2d); then the cards leave
/// the hands together (CR 701.9a).
fn enters(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let opponents: Vec<PlayerId> = g.state().opponents(item.controller).collect();
    let mut discarded = Vec::new();
    for opponent in opponents {
        discarded.extend(choose_discard_two_unless_discarding(g, opponent, item.object, |chars| !chars.is(CardType::Land))?);
    }
    g.discard_together(&discarded);
    Ok(())
}

/// "At the beginning of each opponent's upkeep", once the Class is level 2 (CR 716.2a).
fn opponents_upkeep_at_level_two(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    if class::level(q, me.source) >= 2 { triggers::each_opponents_upkeep(q, me, event) } else { None }
}

/// "At the beginning of your draw step", once the Class is level 3 (CR 716.2a).
fn your_draw_step_at_level_three(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    if class::level(q, me.source) >= 3 { triggers::your_draw_step(q, me, event) } else { None }
}

/// "Has one or fewer cards in hand."
fn has_one_or_fewer_cards(q: &dyn Query, player: PlayerId) -> bool {
    q.state().zone(Zone::hand(player)).len() <= 1
}

/// "If that player has one or fewer cards in hand" (CR 603.4): the player whose upkeep it is, checked as the step
/// begins and again as the ability resolves.
fn active_player_has_one_or_fewer_cards(q: &dyn Query, _: &Asker) -> bool {
    has_one_or_fewer_cards(q, q.state().turn.active)
}

/// "They lose 2 life."
fn lose_two_life(g: &mut dyn Game, _: &StackItem) -> Result<(), Suspend> {
    let player = g.state().turn.active;
    g.lose_life(player, 2);
    Ok(())
}

/// "Draw an additional card for each opponent who has one or fewer cards in hand."
fn draw_for_each_opponent_with_one_or_fewer(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let count = g.state().opponents(item.controller).filter(|&p| has_one_or_fewer_cards(g, p)).count();
    g.draw(item.controller, count as u32);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, enters, "When this Class enters, each opponent discards two cards unless they discard a nonland card."),
        class::level_two("{B}", "{B}: Level 2"),
        triggered(
            when_asked(EventMask::STEP_BEGAN, opponents_upkeep_at_level_two),
            lose_two_life,
            "At the beginning of each opponent's upkeep, if that player has one or fewer cards in hand, they lose 2 life.",
        )
        .condition(active_player_has_one_or_fewer_cards),
        class::level_three("{3}{B}", "{3}{B}: Level 3"),
        triggered(
            when_asked(EventMask::STEP_BEGAN, your_draw_step_at_level_three),
            draw_for_each_opponent_with_one_or_fewer,
            "At the beginning of your draw step, draw an additional card for each opponent who has one or fewer cards in hand.",
        ),
    ],
    labels: &["Discard a nonland card", "Discard two cards"],
    ..oracle::BANDITS_TALENT
};
