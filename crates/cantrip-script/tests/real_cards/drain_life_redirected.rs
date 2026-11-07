//! Drain Life's "but not more life than the player's life total before the damage was dealt, the planeswalker's loyalty before
//! the damage was dealt" for damage redirected (CR 614.9) to a recipient the spell never named. A planeswalker loses no more
//! than the loyalty counters it has (CR 120.3c), and a player who takes the damage as poison counters (CR 120.3b) loses no life,
//! so what the recipient had before the damage is the cap, not what it has after plus the damage dealt. No printed card in the
//! engine redirects damage to a player or a planeswalker (Harm's Way is not in it), so test cards do.

use std::sync::OnceLock;

use cantrip_api::{Asker, Proposal, Query, ReplacementDef, ReplacementKind, replacement};
use cantrip_engine::*;

/// "{T}: Add {B}." - four of them pay for Drain Life with X = 2.
static FONT: CardDef = CardDef {
    name: "Probe Font",
    types: TypeSet::of(&[CardType::Artifact]),
    abilities: &[mana_ability(TAP, add(Mana::B), "{T}: Add {B}.")],
    ..CardDef::BLANK
};

/// A 1/3 creature that costs nothing.
static ANGEL: CardDef = CardDef {
    name: "Probe Angel",
    types: TypeSet::of(&[CardType::Creature]),
    power: Some(1),
    toughness: Some(3),
    ..CardDef::BLANK
};

/// A planeswalker with 1 loyalty that costs nothing.
static WALKER: CardDef = CardDef { name: "Probe Walker", types: TypeSet::of(&[CardType::Planeswalker]), loyalty: Some(1), ..CardDef::BLANK };

fn damage_to_bot2(_: &dyn Query, _: &Asker, proposal: &Proposal) -> bool {
    matches!(proposal, Proposal::Damage(d) if d.amount > 0 && d.target == Target::Player(PlayerId(1)))
}

/// "If damage would be dealt to Bot2, it is dealt to Probe Walker instead."
fn to_the_walker(q: &dyn Query, _: &Asker, proposal: &mut Proposal) {
    if let Proposal::Damage(d) = proposal
        && let Some(&walker) = q.state().battlefield().iter().find(|&&id| q.chars(id).types.has(CardType::Planeswalker))
    {
        d.target = Target::Object(walker);
    }
}

const WALKER_REDIRECT: ReplacementDef = replacement(ReplacementKind::Damage, damage_to_bot2, to_the_walker, "damage to Bot2 is dealt to the walker instead");

/// "Damage dealt to a creature is dealt to Bot2 instead."
fn damage_to_a_creature(q: &dyn Query, _: &Asker, proposal: &Proposal) -> bool {
    matches!(proposal, Proposal::Damage(d) if d.amount > 0 && matches!(d.target, Target::Object(id) if q.chars(id).types.has(CardType::Creature)))
}

fn to_bot2(_: &dyn Query, _: &Asker, proposal: &mut Proposal) {
    if let Proposal::Damage(d) = proposal {
        d.target = Target::Player(PlayerId(1));
    }
}

const PLAYER_REDIRECT: ReplacementDef = replacement(ReplacementKind::Damage, damage_to_a_creature, to_bot2, "damage to a creature is dealt to Bot2 instead");

/// "All damage is dealt to Bot2 as though its source had infect" (CR 702.90b): poison counters, no life loss.
fn infect_for_bot2(_: &dyn Query, _: &Asker, player: PlayerId) -> bool {
    player == PlayerId(1)
}

static WALKER_CONDUIT: CardDef = CardDef { name: "Probe Walker Conduit", types: TypeSet::of(&[CardType::Artifact]), replacements: &[WALKER_REDIRECT], ..CardDef::BLANK };
static PLAYER_CONDUIT: CardDef = CardDef {
    name: "Probe Player Conduit",
    types: TypeSet::of(&[CardType::Artifact]),
    replacements: &[PLAYER_REDIRECT],
    infect_damage_to: &[infect_for_bot2],
    ..CardDef::BLANK
};

fn table() -> &'static CardTable {
    static TABLE: OnceLock<CardTable> = OnceLock::new();
    TABLE.get_or_init(|| CardTable::new(cantrip_cards::all().into_iter().chain([&FONT, &ANGEL, &WALKER, &WALKER_CONDUIT, &PLAYER_CONDUIT])))
}

/// Bot1 casts `hand` in order (Drain Life last, with X = 2, at `target` when it is a name of a permanent on the battlefield, else
/// at Bot2), with Bot2 on `bot2_life`. Bot1's life, Bot2's life and Bot2's poison counters once everything has resolved.
fn play(hand: &[&str], target_name: Option<&str>, bot2_life: i32) -> (i32, i32, u8) {
    let names = |list: &[&str]| -> Vec<CardId> { list.iter().map(|n| table().find(n).unwrap_or_else(|| panic!("no card {n}"))).collect() };
    let deck = |hand: &[&str]| {
        let mut deck = names(hand);
        deck.extend(names(&vec!["Island"; 30 - hand.len()]));
        deck
    };
    let mut game = Game::new(table(), &Setup { decks: vec![deck(hand), deck(&[])], seed: 1, shuffle: false, opening_chooser: None, ..Default::default() });
    game.verify = true;
    game.state.players[1].life = bot2_life;
    let mut casts = hand.iter().copied().peekable();
    for _ in 0..3000 {
        game.run();
        // Done once the last spell has been paid for and resolved: Bot1 has priority again with nothing waiting.
        let idle = matches!(game.pending().map(|r| &r.kind), Some(RequestKind::Priority { .. }));
        if casts.peek().is_none() && idle && game.state.stack.is_empty() && game.state.triggers.is_empty() {
            let bot2 = game.state.player(PlayerId(1));
            return (game.state.player(PlayerId(0)).life, bot2.life, bot2.poison());
        }
        let Some(request) = game.pending().cloned() else { panic!("the game ended") };
        let answer = match &request.kind {
            RequestKind::Priority { options } if request.player == PlayerId(0) => {
                let option = casts.peek().and_then(|&name| options.iter().find(|o| o.kind == ActionKind::CastSpell && o.source.is_some_and(|c| game.name(c) == name)));
                match option {
                    Some(option) => {
                        casts.next();
                        let mut action = Action::of(option);
                        if option.targets_required > 0 {
                            action.targets = vec![match target_name {
                                Some(name) => Target::Object(*game.state.battlefield().iter().find(|&&id| game.name(id) == name).expect("the target")),
                                None => Target::Player(PlayerId(1)),
                            }];
                        }
                        Answer::Act(action)
                    }
                    None => Answer::Pass,
                }
            }
            RequestKind::Priority { .. } => Answer::Pass,
            RequestKind::PayCosts { announcing: true, .. } => Answer::Announce(2),
            RequestKind::PayCosts { options, .. } => {
                let font = options.iter().find(|o| o.kind == ActionKind::ActivateAbility);
                let confirm = options.iter().find(|o| o.kind == ActionKind::ConfirmCasting);
                Answer::Act(Action::of(confirm.or(font).expect("mana to pay with, or the payment to confirm")))
            }
            RequestKind::OrderTriggers { sources, .. } => Answer::Order((0..count_u32(sources.len())).collect()),
            RequestKind::DiscardToHandSize { count, hand } => Answer::Cards(hand.iter().rev().copied().take(*count as usize).collect()),
            RequestKind::DeclareAttackers { .. } => Answer::Attack(Vec::new()),
            RequestKind::DeclareBlockers { .. } => Answer::Block(Vec::new()),
            other => panic!("unexpected question {other:?}"),
        };
        game.answer(request.id, request.player, answer).unwrap();
    }
    panic!("the game went on too long");
}

/// Drain Life X = 2 at Bot2, the damage redirected to a planeswalker with 1 loyalty: all 2 damage is dealt to it (CR 120.3c
/// removes 1 loyalty counter, the planeswalker losing no more than it has), but the loyalty before the damage was 1, so Bot1
/// gains 1, not the 2 that the walker's 0 loyalty after plus the damage dealt would give.
#[test]
fn damage_redirected_to_a_planeswalker_gains_no_more_than_its_loyalty_before() {
    let (life, bot2, poison) = play(&["Probe Font", "Probe Font", "Probe Font", "Probe Font", "Probe Walker", "Probe Walker Conduit", "Drain Life"], None, 20);
    assert_eq!((life, bot2, poison), (21, 20, 0));
}

/// Drain Life X = 2 at a creature, the damage redirected to Bot2 on 1 life, who takes all damage as poison counters (CR 120.3b,
/// 702.90b): 2 damage is dealt, Bot2 loses no life and gets 2 poison counters, and the life total before the damage was 1, so
/// Bot1 gains 1, not the 2 that Bot2's 1 life after plus the damage dealt would give.
#[test]
fn damage_redirected_to_a_player_who_takes_poison_gains_no_more_than_their_life_before() {
    let (life, bot2, poison) = play(&["Probe Font", "Probe Font", "Probe Font", "Probe Font", "Probe Angel", "Probe Player Conduit", "Drain Life"], Some("Probe Angel"), 1);
    assert_eq!((life, bot2, poison), (21, 1, 2));
}
