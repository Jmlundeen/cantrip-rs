use cantrip_api::*;
use cantrip_mechanics::{effects, triggers};
use cantrip_oracle as oracle;

const MAY_DRAW: Question = Question::new(
    "Draw",
    "draw a card?",
    "If you do, you can't attack this turn",
    "Draw",
    "Don't draw",
);

/// "That player may draw a card. If they do, they can't attack this turn and you put a peace counter on this enchantment."
/// The draw is offered to that player alone; a card drawn is what makes the rest happen (CR 608.2c), not the yes - a draw a
/// replacement took (CR 614.6) or an empty library stops is none - and the rule that keeps them from attacking is about the
/// creatures they control as it is read (CR 611.2c), not only those they have now.
fn draw_for_peace(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Target::Player(that_player) = item.subject else { return Ok(()) };
    let before = g.history().drawn(that_player);
    if effects::may_draw(g, that_player, item.object, 1, &MAY_DRAW)? && g.history().drawn(that_player) > before {
        g.end_sentence();
        g.add_lasting_rule(item, 0, Target::Player(that_player), Duration::EndOfTurn);
        g.end_sentence();
        g.add_counters(item.object, CounterKind::Peace, 1);
    }
    Ok(())
}

/// "That player can't attack this turn" (CR 508.1c): the creatures of the player the effect is about.
fn that_players_creatures(q: &dyn Query, _: &Asker, about: Target, id: ObjectId) -> bool {
    let Target::Player(player) = about else { return false };
    q.chars(id).is(CardType::Creature) && q.controller(id) == player
}

/// "At the beginning of your upkeep, if this enchantment has twelve or more peace counters on it, you win the game." The
/// intervening "if" is read as the ability triggers and again as it resolves (CR 603.4).
fn twelve_peace(q: &dyn Query, asker: &Asker) -> bool {
    q.state().obj(asker.source).counters(CounterKind::Peace) >= 12
}

fn win_the_game(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.win_game(item.controller);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            triggers::EACH_PLAYERS_BEGINNING_OF_COMBAT,
            draw_for_peace,
            "At the beginning of combat on each player's turn, that player may draw a card. If they do, they can't attack this turn and you put a peace counter on this enchantment.",
        ),
        triggered(
            triggers::YOUR_UPKEEP,
            win_the_game,
            "At the beginning of your upkeep, if this enchantment has twelve or more peace counters on it, you win the game.",
        )
        .condition(twelve_peace),
    ],
    lasting_rules: &[LastingRule {
        text: "That player can't attack this turn.",
        rule: LastingRuleKind::CantAttack,
        applies: that_players_creatures,
    }],
    ..oracle::PEACE_IN_OUR_TIME
};
