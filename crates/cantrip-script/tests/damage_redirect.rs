//! Damage redirected to a planeswalker (CR 614.1a, 614.6): `ops::deal_damage_with` judges loyalty against the recipient the
//! replacement effects leave, so the redirected damage removes loyalty counters (CR 120.3c) and the player it was meant for loses
//! no life. The printed redirections in the engine (Ancient Adamantoise, Palisade Giant) send damage to a creature, so a test
//! planeswalker with their replacement does here.

use std::sync::OnceLock;

use cantrip_api::Game as Rules;
use cantrip_engine::*;
use cantrip_mechanics::replacements::DAMAGE_TO_YOU_AND_YOUR_OTHER_PERMANENTS_IS_DEALT_TO_THIS;

/// A zero-cost planeswalker of loyalty 5: "All damage that would be dealt to you and other permanents you control is dealt to
/// this planeswalker instead."
static WALKER: CardDef = CardDef {
    name: "Probe Shieldwalker",
    types: TypeSet::of(&[CardType::Planeswalker]),
    loyalty: Some(5),
    replacements: &[DAMAGE_TO_YOU_AND_YOUR_OTHER_PERMANENTS_IS_DEALT_TO_THIS],
    ..CardDef::BLANK
};

fn player(_: &dyn Query, _: &Asker, t: Target) -> bool {
    matches!(t, Target::Player(_))
}

/// "Probe Zap deals 3 damage to target player."
fn zap(g: &mut dyn Rules, item: &StackItem) -> Result<(), Suspend> {
    g.damage(item.object, item.targets()[0], 3);
    Ok(())
}

static ZAP: CardDef = CardDef { name: "Probe Zap", types: TypeSet::of(&[CardType::Instant]), spell: spell(&[target(player)], zap), ..CardDef::BLANK };

static FILLER: CardDef = CardDef { name: "Filler", has_cost: false, types: TypeSet::of(&[CardType::Land]), ..CardDef::BLANK };

fn table() -> &'static CardTable {
    static TABLE: OnceLock<CardTable> = OnceLock::new();
    TABLE.get_or_init(|| CardTable::new([&WALKER, &ZAP, &FILLER]))
}

/// Bot1 casts the Shieldwalker on turn 1; on turn 2 Bot2 zaps Bot1 for 3, which is dealt to the Shieldwalker instead.
#[test]
fn damage_redirected_to_a_planeswalker_removes_its_loyalty() {
    let deck = |first: &str| {
        let mut deck = vec![table().find(first).unwrap()];
        deck.extend(std::iter::repeat_n(table().find("Filler").unwrap(), 29));
        deck
    };
    let mut game = Game::new(table(), &Setup { decks: vec![deck("Probe Shieldwalker"), deck("Probe Zap")], seed: 1, shuffle: false, opening_chooser: None, ..Default::default() });
    game.verify = true;
    let mut zapped = false;
    for _ in 0..300 {
        game.run();
        if zapped && game.state.stack.is_empty() {
            break;
        }
        let request = game.pending().cloned().expect("the game goes on");
        let cast = |options: &[ActionOption], name: &str| options.iter().find(|o| o.kind == ActionKind::CastSpell && o.source.is_some_and(|s| game.name(s) == name)).cloned();
        let answer = match &request.kind {
            RequestKind::Priority { options } if request.player == PlayerId(0) => cast(options, "Probe Shieldwalker").map_or(Answer::Pass, |o| Answer::Act(Action::of(&o))),
            RequestKind::Priority { options } if game.state.turn.number == 2 && !zapped && game.state.stack.is_empty() => match cast(options, "Probe Zap") {
                Some(option) => {
                    zapped = true;
                    let mut action = Action::of(&option);
                    action.targets = vec![Target::Player(PlayerId(0))];
                    Answer::Act(action)
                }
                None => Answer::Pass,
            },
            RequestKind::Priority { .. } => Answer::Pass,
            RequestKind::DiscardToHandSize { count, hand } => Answer::Cards(hand.iter().copied().take(*count as usize).collect()),
            RequestKind::DeclareAttackers { .. } => Answer::Attack(Vec::new()),
            RequestKind::DeclareBlockers { .. } => Answer::Block(Vec::new()),
            other => panic!("unexpected question {other:?}"),
        };
        game.answer(request.id, request.player, answer).unwrap();
    }
    assert!(zapped, "Bot2 never zapped Bot1");
    let walker = *game.state.battlefield().iter().find(|&&id| game.name(id) == "Probe Shieldwalker").expect("the Shieldwalker");
    assert_eq!(game.state.obj(walker).counters(CounterKind::Loyalty), 2, "the redirected damage removed no loyalty");
    assert_eq!(game.state.player(PlayerId(0)).life, 20, "Bot1 lost life to the redirected damage");
}
