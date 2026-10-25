//! Which permission a spell with a cleanup rider is cast by (CR 601.3; Necromancy's 2022-12-08 ruling), where another
//! permission would let it be cast at a time a sorcery couldn't have been: that other permission is judged on the spell as it
//! is on the stack under its caster (CR 601.2a, 601.3b), not on the card where it was. A test permission reads the spell's
//! zone, standing in for a quality the spell has only there (Kheru Mind-Eater's exiled Necromancy cast under Encroaching
//! Mycosynth is an artifact, so historic, so Raff Capashen's permission applies) - no implemented chain reaches it.

use crate::{Game, Setup};
use cantrip_api::*;

const YOU: PlayerId = PlayerId(0);
const THEM: PlayerId = PlayerId(1);

fn nothing(_: &mut dyn cantrip_api::Game, _: &StackItem) -> Result<(), Suspend> {
    Ok(())
}

/// "You may cast this spell as though it had flash. If you cast it any time a sorcery couldn't have been cast, ..."
static RIDER: CardDef = CardDef {
    name: "Cast Permission Test Rider",
    types: TypeSet::of(&[CardType::Enchantment]),
    cast_as_though_flash: true,
    untimely_cleanup_rider: true,
    spell: spell(&[], nothing),
    ..CardDef::BLANK
};

/// "You may cast spells you control on the stack as though they had flash": true only of the spell as it is on the stack.
fn on_the_stack(q: &dyn Query, asker: &Asker, spell: ObjectId, caster: PlayerId) -> bool {
    caster == asker.controller && q.obj(spell).zone.is(ZoneKind::Stack)
}
static GRANTER: CardDef = CardDef {
    name: "Cast Permission Test Granter",
    types: TypeSet::of(&[CardType::Enchantment]),
    flash_rules: &[on_the_stack],
    ..CardDef::BLANK
};
static LAND: CardDef = CardDef { name: "Cast Permission Test Land", has_cost: false, types: TypeSet::of(&[CardType::Land]), ..CardDef::BLANK };

/// The first question after you cast the rider card in your opponent's precombat main phase, the granter yours.
fn asked_after_casting() -> Request {
    let table: &'static CardTable = Box::leak(Box::new(CardTable::new([&RIDER, &GRANTER, &LAND])));
    let filler = table.id_of(&LAND);
    let mut game = Game::new(table, &Setup { decks: vec![vec![filler; 30], vec![filler; 30]], ..Default::default() });
    let put = |game: &mut Game, def: &'static CardDef, zone: Zone| {
        let id = game.state.push_obj(Obj::new(table.id_of(def), YOU, zone));
        game.state.zone_push(zone, id);
        id
    };
    let card = put(&mut game, &RIDER, Zone::hand(YOU));
    put(&mut game, &GRANTER, Zone::BATTLEFIELD);
    for _ in 0..200 {
        game.run();
        let r = game.pending().cloned().expect("a question");
        if r.player == YOU && game.state.turn.active == THEM && game.state.turn.step() == Step::PrecombatMain {
            let RequestKind::Priority { options } = &r.kind else { panic!("{:?}", r.kind) };
            let option = options.iter().find(|o| o.kind == ActionKind::CastSpell && o.source == Some(card)).expect("the rider card offered");
            game.answer(r.id, r.player, Answer::Act(Action::of(option))).unwrap();
            game.run();
            return game.pending().cloned().expect("a question");
        }
        let answer = match &r.kind {
            RequestKind::Priority { .. } => Answer::Pass,
            RequestKind::DeclareAttackers { .. } => Answer::Attack(Vec::new()),
            RequestKind::DeclareBlockers { .. } => Answer::Block(Vec::new()),
            RequestKind::DiscardToHandSize { count, hand } => Answer::Cards(hand.iter().copied().take(*count as usize).collect()),
            other => panic!("unexpected {other:?}"),
        };
        game.answer(r.id, r.player, answer).unwrap();
    }
    panic!("the opponent's main phase never came");
}

#[test]
fn another_flash_permission_is_judged_on_the_spell_on_the_stack() {
    let r = asked_after_casting();
    let RequestKind::ChooseModes { modes, .. } = &r.kind else { panic!("no choice of permission was asked: {:?}", r.kind) };
    assert_eq!(modes.len(), 2, "{modes:?}");
    assert_eq!(modes[0], Label::CastPermission { own: true, by: None });
    assert!(matches!(modes[1], Label::CastPermission { own: false, by: Some(_) }), "{modes:?}");
}
