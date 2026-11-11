//! Mana two "when that mana is spent" delayed abilities ride (CR 603.7a), from one source, crosses the host protocol as two
//! kinds of mana: the view shows each rider's mana apart, the payment offers each as its own `AssignMana` option naming its
//! rider, an answer taking one by its `OptionId` spends that rider (CR 601.2h) and round-trips to the same option, and an
//! answer naming no option is refused, since the options alike in all the wire says differ in their riders.
use cantrip_engine::*;
use cantrip_host::{handles::HandleSpace, prompt::prompt, protocol::*, view::player_view};
use std::collections::HashSet;
const YOU: PlayerId = PlayerId(0);
static LAND: CardDef = CardDef { name: "Rider wire source", types: TypeSet::of(&[CardType::Land]), ..CardDef::BLANK };
fn nothing(_: &mut dyn cantrip_api::Game, _: &StackItem) -> Result<(), Suspend> { Ok(()) }
static SPELL: CardDef = CardDef { name: "Rider wire spell", cost: mana("{R}"), has_cost: true, types: TypeSet::of(&[CardType::Sorcery]), spell: spell(&[], nothing), ..CardDef::BLANK };

#[test]
fn two_riders_of_one_source_cross_the_wire_apart_and_each_answer_spends_its_own() {
    let cards = Box::leak(Box::new(CardTable::new([&LAND, &SPELL])));
    let mut g = Game::new(cards, &Setup { decks: vec![vec![cards.id_of(&LAND); 20]; 2], ..Default::default() });
    while !g.state.agenda.is_empty() { g.state.frame_remove(0); }
    g.state.update_turn(|t| { t.active = YOU; t.priority = YOU; t.set_step(Step::PrecombatMain); });
    g.state.frame_push(Frame::new(FrameKind::Priority));
    let source = g.state.zone(Zone::library(YOU))[0];
    let spell = g.state.push_obj(Obj::new(cards.id_of(&SPELL), YOU, Zone::hand(YOU)));
    g.state.zone_push(Zone::hand(YOU), spell);
    g.state.add_mana_riding(YOU, Mana::R, 1, source, None, ManaLasts::Step, false, 7);
    g.state.add_mana_riding(YOU, Mana::R, 1, source, None, ManaLasts::Step, false, 9);
    g.run();
    let request = g.pending().unwrap().clone();
    let RequestKind::Priority { options } = &request.kind else { panic!("priority") };
    let action = Action::of(options.iter().find(|o| o.kind == ActionKind::CastSpell && o.source == Some(spell)).unwrap());
    g.answer(request.id, YOU, Answer::Act(action)).unwrap();
    g.run();
    let mut handles = HandleSpace::new();
    let empty = HashSet::new();
    let view = player_view(&g, YOU, &mut handles, &["one".into(), "two".into()], &[false; 2], &empty, &empty, &empty);
    let me = &view.players[0];
    assert_eq!(me.mana, [0; 6], "neither rider's mana is shown as ordinary mana");
    let mut shown: Vec<(u32, i32)> = me.restricted_mana.iter().map(|m| (m.rider, m.amount)).collect();
    shown.sort();
    assert_eq!(shown, vec![(7, 1), (9, 1)], "each rider's mana is shown apart, named by its rider");
    let request = g.pending().unwrap().clone();
    let PromptPayload::PayCostsPromptMessage(pay) = prompt(&g, &request, &mut handles).unwrap().0 else { panic!("pay") };
    let choices: Vec<_> = pay.options.iter().filter(|o| o.kind == ActionKindView::AssignMana).collect();
    let mut riders: Vec<u32> = choices.iter().map(|o| o.rider).collect();
    riders.sort();
    assert_eq!(riders, vec![7, 9], "each rider is an option of its own, naming its rider");
    for option in &choices {
        let wire = TakeActionAnswer::take(option);
        let engine = cantrip_host::answer::translate(&AnswerPayload::TakeActionAnswer(wire.clone()), &request, &handles, &g)
            .unwrap_or_else(|_| panic!("wire translates"));
        let Answer::Act(act) = &engine else { panic!("an action") };
        assert_eq!(act.rider, option.rider, "the answer spends the rider its option names");
        assert!(cantrip_engine::check_answer(&g, &request, &engine).is_ok());
        let AnswerPayload::TakeActionAnswer(back) = cantrip_host::answer::payload_of(&engine, &request, &mut handles, &g) else { panic!("an action") };
        assert_eq!(back.option_id, option.option_id, "and round-trips to its own option");
        let mut unnamed = wire.clone();
        unnamed.option_id = 0;
        assert!(
            cantrip_host::answer::translate(&AnswerPayload::TakeActionAnswer(unnamed), &request, &handles, &g).is_err(),
            "without its OptionId the answer could be either rider's, and is refused"
        );
    }
}
