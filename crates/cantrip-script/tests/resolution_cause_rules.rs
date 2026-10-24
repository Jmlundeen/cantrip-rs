//! A cost a resolving spell or ability has a player pay (CR 118.12) under "spells and abilities your opponents control can't
//! cause you to discard cards or sacrifice permanents" (CR 101.2; Tamiyo, Collector of Tales, Sigarda, Host of Herons): an
//! opponent's "unless they sacrifice a creature" or "unless they discard a card" can't be paid - the payment is not offered
//! (Tamiyo's ruling: "something happens unless you sacrifice" can't be paid) - so what it is the price of happens. Without the
//! static ability it is paid (the positive control). On a table of test-only cards: no printed card in the engine asks an
//! opponent for such a payment.

use std::sync::OnceLock;

use cantrip_api::Game as Rules;
use cantrip_engine::*;

/// "Spells and abilities your opponents control can't cause you to discard cards or sacrifice permanents."
fn opponents_cant_cause_you(_: &dyn Query, asker: &Asker, causer: PlayerId, player: PlayerId, kind: CauseKind) -> bool {
    !(player == asker.controller && causer != player && matches!(kind, CauseKind::Sacrifice | CauseKind::Discard))
}

static SHIELD: CardDef = CardDef {
    name: "Probe Shield",
    types: TypeSet::of(&[CardType::Artifact]),
    cause_rules: &[opponents_cant_cause_you],
    ..CardDef::BLANK
};

fn creature(q: &dyn Query, _: &Asker, id: ObjectId) -> bool {
    q.chars(id).is(CardType::Creature)
}

const PAY: Question = Question::new("PayToAvoid", "pay {cost}?", "", "Pay {cost}", "Don't");

/// Each opponent loses 5 life unless they pay `cost`.
fn unless_each_opponent_pays(g: &mut dyn Rules, item: &StackItem, cost: Cost) -> Result<(), Suspend> {
    for opponent in g.state().opponents(item.controller).collect::<Vec<_>>() {
        if !g.may_pay(opponent, item.object, item.object, cost, &PAY)? {
            g.lose_life(opponent, 5);
        }
    }
    Ok(())
}

fn unless_sacrifice(g: &mut dyn Rules, item: &StackItem) -> Result<(), Suspend> {
    unless_each_opponent_pays(g, item, Cost::sacrifice(1, creature))
}

fn unless_discard(g: &mut dyn Rules, item: &StackItem) -> Result<(), Suspend> {
    unless_each_opponent_pays(g, item, Cost { discard: Discards::one(1, None), ..Cost::NONE })
}

static TAX: CardDef = CardDef {
    name: "Probe Tax",
    types: TypeSet::of(&[CardType::Artifact]),
    abilities: &[
        ability(Cost::NONE, unless_sacrifice, "Each opponent loses 5 life unless they sacrifice a creature."),
        ability(Cost::NONE, unless_discard, "Each opponent loses 5 life unless they discard a card."),
    ],
    ..CardDef::BLANK
};

static BEAR: CardDef = CardDef {
    name: "Probe Bear",
    types: TypeSet::of(&[CardType::Creature]),
    power: Some(2),
    toughness: Some(2),
    ..CardDef::BLANK
};

static FILLER: CardDef = CardDef { name: "Filler", has_cost: false, types: TypeSet::of(&[CardType::Land]), ..CardDef::BLANK };

fn table() -> &'static CardTable {
    static TABLE: OnceLock<CardTable> = OnceLock::new();
    TABLE.get_or_init(|| CardTable::new([&SHIELD, &TAX, &BEAR, &FILLER]))
}

/// Bot1 casts `bot1` on turn 1; on turn 2 Bot2 casts Probe Tax and activates its ability `index`. Bot1 pays whenever paying
/// is offered, with the first card or permanent it may. The game, and whether paying was offered.
fn play(bot1: &[&str], index: u8) -> (Game, bool) {
    let card = |n: &str| table().find(n).unwrap();
    let mut first: Vec<CardId> = bot1.iter().map(|n| card(n)).collect();
    first.extend(std::iter::repeat(card("Filler")).take(30 - first.len()));
    let mut second = vec![card("Probe Tax")];
    second.extend(std::iter::repeat(card("Filler")).take(29));
    let mut game = Game::new(table(), &Setup { decks: vec![first, second], seed: 1, shuffle: false, opening_chooser: None, ..Default::default() });
    game.verify = true;
    let mut casts = bot1.iter().peekable();
    let (mut tax_cast, mut activated, mut offered) = (false, false, None);
    for _ in 0..500 {
        game.run();
        if activated && game.state.stack.is_empty() && game.pending().is_some_and(|r| matches!(r.kind, RequestKind::Priority { .. })) {
            break;
        }
        let request = game.pending().cloned().expect("the game goes on");
        let named = |o: &&ActionOption, kind: ActionKind, name: &str| o.kind == kind && o.source.is_some_and(|s| game.name(s) == name);
        let answer = match &request.kind {
            RequestKind::Priority { options } if game.state.stack.is_empty() && request.player == PlayerId(0) => {
                match casts.peek().and_then(|&&n| options.iter().find(|o| named(o, ActionKind::CastSpell, n))) {
                    Some(o) => {
                        casts.next();
                        Answer::Act(Action::of(o))
                    }
                    None => Answer::Pass,
                }
            }
            RequestKind::Priority { options } if game.state.stack.is_empty() && game.state.turn.number == 2 && !activated => {
                let option = if tax_cast {
                    options.iter().find(|o| o.kind == ActionKind::ActivateAbility && o.index == index && named(o, ActionKind::ActivateAbility, "Probe Tax"))
                } else {
                    options.iter().find(|o| named(o, ActionKind::CastSpell, "Probe Tax"))
                };
                match option {
                    Some(o) => {
                        if tax_cast {
                            activated = true;
                        }
                        tax_cast = true;
                        Answer::Act(Action::of(o))
                    }
                    None => Answer::Pass,
                }
            }
            RequestKind::Priority { .. } => Answer::Pass,
            RequestKind::UnlessPays { options, .. } => {
                let pay = options.iter().find(|o| o.kind == ActionKind::PayCost);
                offered = Some(pay.is_some());
                match pay.or_else(|| options.iter().find(|o| o.kind == ActionKind::DeclineCost)) {
                    Some(o) => Answer::Act(Action::of(o)),
                    None => Answer::Pass,
                }
            }
            RequestKind::ChooseCards { cards, rule, .. } => {
                let choosable: Vec<ObjectId> = cards.iter().copied().filter(|&c| rule.listed_card(c)).collect();
                Answer::Cards(choosable.into_iter().take(1).collect())
            }
            RequestKind::DiscardToHandSize { count, hand } => Answer::Cards(hand.iter().copied().take(*count as usize).collect()),
            RequestKind::DeclareAttackers { .. } => Answer::Attack(Vec::new()),
            RequestKind::DeclareBlockers { .. } => Answer::Block(Vec::new()),
            other => panic!("unexpected question {other:?}"),
        };
        game.answer(request.id, request.player, answer).unwrap();
    }
    assert!(activated, "Bot2 activated Probe Tax");
    (game, offered.unwrap_or(false))
}

fn bear_in_graveyard(game: &Game) -> bool {
    game.state.zone(Zone::graveyard(PlayerId(0))).iter().any(|&c| game.name(c) == "Probe Bear")
}

/// CR 101.2, 118.3: under the Shield the sacrifice Bot2's ability asks for can't be paid, so it is not offered, the Bear stays
/// and Bot1 loses 5 life; without the Shield it is offered and paid.
#[test]
fn an_opponents_unless_sacrifice_cant_be_paid_under_the_shield() {
    let (game, offered) = play(&["Probe Shield", "Probe Bear"], 0);
    assert!(!offered);
    assert!(!bear_in_graveyard(&game));
    assert_eq!(game.state.player(PlayerId(0)).life, 15);
    let (game, offered) = play(&["Probe Bear"], 0);
    assert!(offered);
    assert!(bear_in_graveyard(&game));
    assert_eq!(game.state.player(PlayerId(0)).life, 20);
}

/// The same for a discard: under the Shield "unless they discard a card" is not offered and Bot1 loses 5 life with its hand
/// whole; without it a card is discarded instead.
#[test]
fn an_opponents_unless_discard_cant_be_paid_under_the_shield() {
    let (game, offered) = play(&["Probe Shield"], 1);
    assert!(!offered);
    assert!(game.state.zone(Zone::graveyard(PlayerId(0))).is_empty());
    assert_eq!(game.state.player(PlayerId(0)).life, 15);
    let (game, offered) = play(&[], 1);
    assert!(offered);
    assert_eq!(game.state.zone(Zone::graveyard(PlayerId(0))).len(), 1);
    assert_eq!(game.state.player(PlayerId(0)).life, 20);
}
