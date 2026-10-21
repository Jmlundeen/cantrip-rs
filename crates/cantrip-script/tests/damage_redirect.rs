//! Damage redirected to a planeswalker or a battle (CR 614.1a, 614.6): `ops::deal_damage_with` judges loyalty and defense against
//! the recipient the replacement effects leave, so the redirected damage removes loyalty or defense counters (CR 120.3c, 120.3h)
//! and the player it was meant for loses no life. The printed redirections in the engine (Ancient Adamantoise, Palisade Giant)
//! send damage to a creature, so test permanents do here.

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

/// A zero-cost Siege of defense 5 (CR 310.4a), with no abilities of its own.
static SIEGE: CardDef = CardDef {
    name: "Probe Siege",
    types: TypeSet::of(&[CardType::Battle]),
    subtypes: SubtypeSet::of(&[Subtype::Siege]),
    defense: Some(5),
    ..CardDef::BLANK
};

/// "If damage would be dealt to you, it is dealt to a battle you control instead."
fn damage_to_you_with_a_battle(q: &dyn Query, asker: &Asker, proposal: &Proposal) -> bool {
    matches!(proposal, Proposal::Damage(d) if d.amount > 0 && d.target == Target::Player(asker.controller))
        && q.permanents_of(asker.controller).into_iter().any(|id| q.chars(id).is(CardType::Battle))
}

fn to_your_battle(q: &dyn Query, asker: &Asker, proposal: &mut Proposal) {
    if let Proposal::Damage(d) = proposal
        && let Some(battle) = q.permanents_of(asker.controller).into_iter().find(|&id| q.chars(id).is(CardType::Battle))
    {
        d.target = Target::Object(battle);
    }
}

/// A zero-cost enchantment: "If damage would be dealt to you, it is dealt to a battle you control instead."
static BULWARK: CardDef = CardDef {
    name: "Probe Bulwark",
    types: TypeSet::of(&[CardType::Enchantment]),
    replacements: &[replacement(ReplacementKind::Damage, damage_to_you_with_a_battle, to_your_battle, "it is dealt to a battle you control instead")],
    ..CardDef::BLANK
};

fn player(_: &dyn Query, _: &Asker, t: Target) -> bool {
    matches!(t, Target::Player(_))
}

/// "Probe Zap deals 3 damage to target player."
fn zap(g: &mut dyn Rules, item: &StackItem) -> Result<(), Suspend> {
    g.damage(item.object, item.target(0), 3);
    Ok(())
}

static ZAP: CardDef = CardDef { name: "Probe Zap", types: TypeSet::of(&[CardType::Instant]), spell: spell(&[target(player)], zap), ..CardDef::BLANK };

static FILLER: CardDef = CardDef { name: "Filler", has_cost: false, types: TypeSet::of(&[CardType::Land]), ..CardDef::BLANK };

fn table() -> &'static CardTable {
    static TABLE: OnceLock<CardTable> = OnceLock::new();
    TABLE.get_or_init(|| CardTable::new([&WALKER, &SIEGE, &BULWARK, &ZAP, &FILLER]))
}

/// Bot1 casts `permanents` on turn 1 (a Siege protected by Bot2); on turn 2 Bot2 zaps Bot1 for 3. The game afterwards.
fn zapped_after(permanents: &[&str]) -> Game {
    let deck = |first: &[&str]| {
        let mut deck: Vec<CardId> = first.iter().map(|n| table().find(n).unwrap()).collect();
        deck.extend(std::iter::repeat_n(table().find("Filler").unwrap(), 30 - first.len()));
        deck
    };
    let mut game = Game::new(table(), &Setup { decks: vec![deck(permanents), deck(&["Probe Zap"])], seed: 1, shuffle: false, opening_chooser: None, ..Default::default() });
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
            RequestKind::Priority { options } if request.player == PlayerId(0) && game.state.stack.is_empty() => {
                permanents.iter().find_map(|n| cast(options, n)).map_or(Answer::Pass, |o| Answer::Act(Action::of(&o)))
            }
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
            RequestKind::ChoosePlayer { candidates, .. } => Answer::Player(candidates[0]),
            RequestKind::DiscardToHandSize { count, hand } => Answer::Cards(hand.iter().copied().take(*count as usize).collect()),
            RequestKind::DeclareAttackers { .. } => Answer::Attack(Vec::new()),
            RequestKind::DeclareBlockers { .. } => Answer::Block(Vec::new()),
            other => panic!("unexpected question {other:?}"),
        };
        game.answer(request.id, request.player, answer).unwrap();
    }
    assert!(zapped, "Bot2 never zapped Bot1");
    game
}

fn named(game: &Game, name: &str) -> ObjectId {
    *game.state.battlefield().iter().find(|&&id| game.name(id) == name).unwrap_or_else(|| panic!("no {name}"))
}

/// The Shieldwalker takes the 3 meant for Bot1 as 3 loyalty.
#[test]
fn damage_redirected_to_a_planeswalker_removes_its_loyalty() {
    let game = zapped_after(&["Probe Shieldwalker"]);
    assert_eq!(game.state.obj(named(&game, "Probe Shieldwalker")).counters(CounterKind::Loyalty), 2, "the redirected damage removed no loyalty");
    assert_eq!(game.state.player(PlayerId(0)).life, 20, "Bot1 lost life to the redirected damage");
}

/// The Bulwark sends the 3 meant for Bot1 to its Siege, which loses 3 defense counters (CR 120.3h, 310.6).
#[test]
fn damage_redirected_to_a_battle_removes_its_defense() {
    let game = zapped_after(&["Probe Siege", "Probe Bulwark"]);
    let siege = named(&game, "Probe Siege");
    assert_eq!(game.state.obj(siege).marks.protector, Some(PlayerId(1)));
    assert_eq!(game.state.obj(siege).counters(CounterKind::Defense), 2, "the redirected damage removed no defense");
    assert_eq!(game.state.player(PlayerId(0)).life, 20, "Bot1 lost life to the redirected damage");
}
