//! A spell that may choose the same mode more than once (CR 700.2d) keeps how many times each mode was chosen: the Seasons
//! choose one mode up to five times. Each mode's count took two bits of the mask, so a fourth choice carried into the next
//! mode's count. Over a test card of the Seasons' shape.

use crate::{Game, Setup};
use cantrip_api::*;
use cantrip_state::{Frame, FrameKind};

const YOU: PlayerId = PlayerId(0);

fn gain_one(g: &mut dyn cantrip_api::Game, item: &StackItem) -> Result<(), Suspend> {
    g.gain_life(item.controller, 1);
    Ok(())
}

fn lose_one(g: &mut dyn cantrip_api::Game, item: &StackItem) -> Result<(), Suspend> {
    g.lose_life(item.controller, 1);
    Ok(())
}

static FILLER: CardDef = CardDef { name: "Mode Count Test Filler", has_cost: false, types: TypeSet::of(&[CardType::Land]), ..CardDef::BLANK };

/// "Choose up to five. You may choose the same mode more than once."
static SEASON_MODES: ModalDef = modes(&[mode(gain_one, "You gain 1 life."), mode(lose_one, "You lose 1 life.")]).choose(1, 5).repeat();

static SEASON: CardDef = CardDef {
    name: "Mode Count Test Season",
    types: TypeSet::of(&[CardType::Sorcery]),
    spell: Some(SpellDef::MODAL),
    modal: Some(&SEASON_MODES),
    ..CardDef::BLANK
};

/// Your main phase with priority and the Season in your hand.
fn game() -> (Game, ObjectId) {
    let table: &'static CardTable = Box::leak(Box::new(CardTable::new([&FILLER, &SEASON])));
    let deck = || vec![table.id_of(&FILLER); 20];
    let mut g = Game::new(table, &Setup { decks: vec![deck(), deck()], ..Default::default() });
    while !g.state.agenda.is_empty() {
        g.state.frame_remove(0);
    }
    g.state.update_turn(|t| {
        t.active = YOU;
        t.priority = YOU;
        t.set_step(Step::PrecombatMain);
    });
    g.state.frame_push(Frame::new(FrameKind::Priority));
    let season = g.state.push_obj(Obj::new(table.id_of(&SEASON), YOU, Zone::hand(YOU)));
    g.state.zone_push(Zone::hand(YOU), season);
    (g, season)
}

fn next(g: &mut Game) -> Request {
    g.run();
    g.pending().cloned().expect("a question")
}

/// Cast with mode 1 chosen five times, it runs five times; with a mode chosen four times and the other once, each runs as
/// often as it was chosen.
#[test]
fn a_mode_chosen_four_or_five_times_runs_that_many_times() {
    for (chosen, gained) in [(vec![0, 0, 0, 0, 0], 5), (vec![0, 0, 0, 0, 1], 3), (vec![0, 1, 1, 1, 1], -3)] {
        let (mut g, season) = game();
        let life = g.state.player(YOU).life;
        let r = next(&mut g);
        let RequestKind::Priority { options } = &r.kind else { panic!("expected priority, got {:?}", r.kind) };
        let cast = options.iter().find(|o| o.kind == ActionKind::CastSpell && o.source == Some(season)).expect("castable").clone();
        g.answer(r.id, r.player, Answer::Act(Action::of(&cast))).expect("a legal answer");
        let r = next(&mut g);
        assert!(matches!(r.kind, RequestKind::ChooseModes { .. }), "the modes are asked, got {:?}", r.kind);
        g.answer(r.id, r.player, Answer::Modes(chosen.clone())).expect("a legal answer");
        for _ in 0..8 {
            let r = next(&mut g);
            if g.state.stack.is_empty() {
                break;
            }
            g.answer(r.id, r.player, Answer::Pass).expect("a legal answer");
        }
        assert!(g.state.stack.is_empty(), "the spell resolved");
        assert_eq!(g.state.player(YOU).life, life + gained, "{chosen:?}");
    }
}
