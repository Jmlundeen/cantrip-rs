//! A static ability whose condition reads the history, granting a triggered ability, and the very event that turns the condition
//! on. CR 603.10: "Normally, objects that exist immediately after an event are checked to see if the event matched any trigger
//! conditions, and continuous effects that exist at that time are used to determine what the trigger conditions are" - so the
//! event is in the history the static ability reads, and the ability it grants triggers on that first event (CR 603.2). Darkblade
//! Agent's shape ("as long as you've surveilled this turn, this creature has ..."); no printed card in the engine grants a
//! triggered ability on a history condition, so a test card does here.

use std::sync::OnceLock;

use cantrip_api::Game as Rules;
use cantrip_engine::*;

/// "Whenever you draw a card": the drawing player is the ability's controller.
fn you_draw(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::Drew { player, .. } if *player == q.controller(source) => Some(Fired { subject: Target::Player(*player), data: [0, 0] }),
        _ => None,
    }
}

fn gain_one(g: &mut dyn Rules, item: &StackItem) -> Result<(), Suspend> {
    g.gain_life(item.controller, 1);
    Ok(())
}

/// "As long as you've drawn a card this turn": the scholar itself, while its controller has.
fn itself_once_you_have_drawn(view: &dyn LayerQuery, source: ObjectId, affected: ObjectId, _: &Chars) -> bool {
    affected == source && view.history().drawn(view.state().obj(source).controller) > 0
}

/// "... this creature has 'Whenever you draw a card, you gain 1 life.'"
fn grant(view: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    let ability = AbilityRef::delayed(view.cards().find("Probe Scholar").expect("the scholar"), 0);
    if let Some(slot) = chars.abilities.iter().position(|a| a.is_none()) {
        chars.abilities[slot] = Some(ability);
    }
}

/// A zero-cost 1/1: "As long as you've drawn a card this turn, this creature has 'Whenever you draw a card, you gain 1 life.'"
static SCHOLAR: CardDef = CardDef {
    name: "Probe Scholar",
    types: TypeSet::of(&[CardType::Creature]),
    power: Some(1),
    toughness: Some(1),
    statics: &[static_ability(Layer::Ability, itself_once_you_have_drawn, grant, "As long as you've drawn a card this turn, this creature has \"Whenever you draw a card, you gain 1 life.\"")],
    delayed: &[triggered(when(EventMask::DREW, you_draw), gain_one, "Whenever you draw a card, you gain 1 life.")],
    ..CardDef::BLANK
};

fn draw_one(g: &mut dyn Rules, item: &StackItem) -> Result<(), Suspend> {
    g.draw(item.controller, 1);
    Ok(())
}

/// A zero-cost instant: "Draw a card."
static PONDER: CardDef = CardDef { name: "Probe Ponder", types: TypeSet::of(&[CardType::Instant]), spell: spell(&[], draw_one), ..CardDef::BLANK };
static FILLER: CardDef = CardDef { name: "Filler", has_cost: false, types: TypeSet::of(&[CardType::Land]), ..CardDef::BLANK };

fn table() -> &'static CardTable {
    static TABLE: OnceLock<CardTable> = OnceLock::new();
    TABLE.get_or_init(|| CardTable::new([&SCHOLAR, &PONDER, &FILLER]))
}

/// Bot1, on turn 1 (no draw step), casts the Scholar and then `ponders` Probe Ponders. Its life afterwards.
fn life_after(ponders: usize) -> i32 {
    let find = |n: &str| table().find(n).unwrap();
    let mut first = vec![find("Probe Scholar")];
    first.extend(std::iter::repeat_n(find("Probe Ponder"), ponders));
    first.extend(std::iter::repeat_n(find("Filler"), 30 - first.len()));
    let mut game = Game::new(table(), &Setup { decks: vec![first, vec![find("Filler"); 30]], seed: 1, shuffle: false, opening_chooser: None, ..Default::default() });
    game.verify = true;
    let mut casts: Vec<&str> = vec!["Probe Scholar"];
    casts.extend(std::iter::repeat_n("Probe Ponder", ponders));
    let mut casts = casts.into_iter().peekable();
    for _ in 0..300 {
        game.run();
        if casts.peek().is_none() && game.state.stack.is_empty() && game.state.triggers.is_empty() {
            break;
        }
        let request = game.pending().cloned().expect("the game goes on");
        let answer = match &request.kind {
            RequestKind::Priority { options } if request.player == PlayerId(0) && game.state.stack.is_empty() => {
                match casts.peek().and_then(|&n| options.iter().find(|o| o.kind == ActionKind::CastSpell && o.source.is_some_and(|s| game.name(s) == n))) {
                    Some(option) => {
                        casts.next();
                        Answer::Act(Action::of(option))
                    }
                    None => Answer::Pass,
                }
            }
            RequestKind::Priority { .. } => Answer::Pass,
            other => panic!("unexpected question {other:?}"),
        };
        game.answer(request.id, request.player, answer).unwrap();
    }
    assert_eq!(game.state.turn.number, 1, "the casts ran past turn 1");
    game.state.player(PlayerId(0)).life
}

/// The first draw of the turn turns the condition on, and the ability it grants triggers on that very draw (CR 603.10); the
/// second draw triggers it again.
#[test]
fn an_ability_granted_by_the_history_triggers_on_the_event_that_granted_it() {
    assert_eq!(life_after(1), 21);
    assert_eq!(life_after(2), 22);
}
