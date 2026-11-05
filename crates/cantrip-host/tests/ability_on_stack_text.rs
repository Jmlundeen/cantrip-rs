//! An ability on the stack as the client draws it (KI-151): its text in the words it kept as it was put on the stack, a
//! text-changing effect on its source since notwithstanding (CR 113.7a, 612.1).

use std::collections::HashSet;

use cantrip_engine::*;
use cantrip_host::handles::HandleSpace;
use cantrip_host::protocol::*;
use cantrip_host::view::player_view;

const YOU: PlayerId = PlayerId(0);

static FILLER: CardDef = CardDef { name: "Stack Text Filler", has_cost: false, types: TypeSet::of(&[CardType::Land]), ..CardDef::BLANK };

fn nothing(_: &mut dyn cantrip_api::Game, _: &StackItem) -> Result<(), Suspend> {
    Ok(())
}

/// "{0}: Elves you control gain trample until end of turn." Its words are changed on the stack: the Elves become Goblins.
static ARTIFACT: CardDef = CardDef {
    name: "Stack Text Artifact",
    types: TypeSet::of(&[CardType::Artifact]),
    abilities: &[ability(Cost::NONE, nothing, "{0}: Elves you control gain trample until end of turn.")],
    ..CardDef::BLANK
};

fn game() -> (Game, ObjectId) {
    let table: &'static CardTable = Box::leak(Box::new(CardTable::new([&FILLER, &ARTIFACT])));
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
    g.state.update_player(YOU, |p| p.turn_began = 1);
    let id = g.state.push_obj(Obj::new(table.id_of(&ARTIFACT), YOU, Zone::BATTLEFIELD));
    g.state.zone_push(Zone::BATTLEFIELD, id);
    (g, id)
}

fn stack_text(g: &Game) -> String {
    let names = vec!["You".to_string(), "Them".to_string()];
    let view = player_view(g, YOU, &mut HandleSpace::new(), &names, &[false, false], &HashSet::new(), &HashSet::new(), &HashSet::new());
    let mut on_stack = view.objects.into_iter().filter(|o| o.kind == ObjectKindView::AbilityOnStack);
    let ability = on_stack.next().expect("the ability is in the view");
    assert!(on_stack.next().is_none(), "one ability");
    ability.oracle_text
}

#[test]
fn an_ability_on_the_stack_is_shown_in_the_words_it_kept() {
    let (mut g, artifact) = game();
    g.run();
    let r = g.pending().cloned().expect("priority");
    let RequestKind::Priority { options } = &r.kind else { panic!("priority") };
    let option = options.iter().find(|o| o.source == Some(artifact)).expect("the ability is offered");
    g.answer(r.id, r.player, Answer::Act(Action::of(option))).expect("a legal answer");
    g.run();
    assert_eq!(g.state.stack.len(), 1, "the ability is on the stack");
    assert_eq!(stack_text(&g), "{0}: Elves you control gain trample until end of turn.");

    let mut item = g.state.stack[0];
    item.words = Words::NONE.changed(Word::Subtype(Subtype::Elf), Word::Subtype(Subtype::Goblin));
    g.state.stack_set(0, item);
    assert_eq!(stack_text(&g), "{0}: Goblins you control gain trample until end of turn.");
}
