//! A permanent's own "enters tapped" and the other replacement effects that modify how it enters are applied in the order
//! the affected object's controller chooses (CR 614.12, 616.1), each judged again on the entry as the ones before left it
//! (CR 616.1f). Spelunking's ruling: "If a land has an ability that says it enters the battlefield tapped, you choose the
//! order in which that ability's effect and Spelunking's effect apply. This means you can choose to have the land enter
//! tapped or untapped." Over test cards: a land that enters tapped, and an enchantment whose "lands enter untapped" any
//! land meets.

use crate::{Game, Setup};
use cantrip_api::*;
use cantrip_state::{Frame, FrameKind};

const YOU: PlayerId = PlayerId(0);
const THEM: PlayerId = PlayerId(1);

/// "This land enters tapped." (CR 614.1d)
fn enters_tapped(_: &mut dyn cantrip_api::Game, entering: &mut Entering) -> Result<(), Suspend> {
    entering.tapped = true;
    Ok(())
}

/// "Lands enter untapped." (CR 614.1d): every land entering, whether it would enter tapped or not as the event stands.
fn a_land_enters(q: &dyn Query, _: &Asker, proposal: &Proposal) -> bool {
    matches!(proposal, Proposal::ZoneChange(z) if z.to.is(ZoneKind::Battlefield) && q.chars(z.object).is(CardType::Land))
}

fn untapped(_: &dyn Query, _: &Asker, proposal: &mut Proposal) {
    if let Proposal::ZoneChange(z) = proposal {
        z.tapped = false;
    }
}

static FILLER: CardDef = CardDef { name: "Entry Order Filler", has_cost: false, types: TypeSet::of(&[CardType::Land]), ..CardDef::BLANK };
static TAPLAND: CardDef = CardDef {
    name: "Entry Order Tapland",
    has_cost: false,
    types: TypeSet::of(&[CardType::Land]),
    text: "This land enters tapped.",
    enters: Some(enters_tapped),
    ..CardDef::BLANK
};
static UNTAPPER: CardDef = CardDef {
    name: "Entry Order Untapper",
    types: TypeSet::of(&[CardType::Enchantment]),
    replacements: &[replacement(ReplacementKind::ZoneChange, a_land_enters, untapped, "lands enter untapped")],
    ..CardDef::BLANK
};

fn game() -> Game {
    let table: &'static CardTable = Box::leak(Box::new(CardTable::new([&FILLER, &TAPLAND, &UNTAPPER])));
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
    g
}

fn put(g: &mut Game, def: &'static CardDef, controller: PlayerId, zone: Zone) -> ObjectId {
    let id = g.state.push_obj(Obj::new(g.cards.id_of(def), controller, zone));
    let timestamp = g.state.timestamp();
    g.state.update_obj(id, |o| o.timestamp = timestamp);
    g.state.zone_push(zone, id);
    g.state.see_statics();
    id
}

/// Plays `land` from your hand and answers each order of replacement effects asked with `first` among the sources offered:
/// whether the land entered tapped, and the sources of each order asked.
fn play(g: &mut Game, land: ObjectId, first: ObjectId) -> (bool, Vec<Vec<ObjectId>>) {
    g.run();
    let r = g.pending().cloned().expect("a question");
    let option = match &r.kind {
        RequestKind::Priority { options } => options.iter().find(|o| o.source == Some(land)).cloned(),
        _ => None,
    }
    .expect("the land play is offered");
    g.answer(r.id, r.player, Answer::Act(Action::of(&option))).expect("a legal answer");
    let mut asked = Vec::new();
    loop {
        g.run();
        let Some(r) = g.pending().cloned() else { break };
        let RequestKind::ChooseReplacement { subject, sources, hints, .. } = &r.kind else { break };
        assert_eq!(r.player, YOU, "the land's controller orders the effects (CR 616.1)");
        assert_eq!(*subject, Some(land));
        let own = sources.iter().position(|&s| s == land).expect("its own replacement is offered");
        assert_eq!(hints[own], "This land enters tapped.", "offered in its own words");
        let pick = sources.iter().position(|&s| s == first).expect("the effect to apply first");
        asked.push(sources.clone());
        g.answer(r.id, r.player, Answer::Index(pick as u8)).expect("a legal answer");
    }
    let new = g.state.obj(land).became.expect("the land entered");
    assert!(g.state.obj(new).zone.is(ZoneKind::Battlefield));
    (g.state.obj(new).tapped, asked)
}

/// Its own "enters tapped" applied first, "lands enter untapped" applies after it, and it enters untapped; the other way
/// round, its own applies last, and it enters tapped. One question, of both effects.
#[test]
fn its_own_enters_tapped_and_another_enter_untapped_apply_in_the_order_its_controller_chooses() {
    for (own_first, tapped) in [(true, false), (false, true)] {
        let mut g = game();
        let untapper = put(&mut g, &UNTAPPER, YOU, Zone::BATTLEFIELD);
        let land = put(&mut g, &TAPLAND, YOU, Zone::hand(YOU));
        let (entered_tapped, asked) = play(&mut g, land, if own_first { land } else { untapper });
        assert_eq!(entered_tapped, tapped, "its own first: {own_first}");
        assert_eq!(asked, vec![vec![land, untapper]], "asked once, of both");
    }
}

/// The effect of an opponent's permanent is ordered by the land's controller all the same (CR 616.1: the affected object's
/// controller).
#[test]
fn the_entering_permanents_controller_orders_an_opponents_effect_with_its_own() {
    let mut g = game();
    let untapper = put(&mut g, &UNTAPPER, THEM, Zone::BATTLEFIELD);
    let land = put(&mut g, &TAPLAND, YOU, Zone::hand(YOU));
    let (tapped, asked) = play(&mut g, land, untapper);
    assert!(tapped);
    assert_eq!(asked.len(), 1);
}

/// Nothing to order with: its own applies, nothing is asked. A land with none of its own meets only the other, unasked.
#[test]
fn nothing_is_asked_where_only_one_effect_applies() {
    let mut g = game();
    let land = put(&mut g, &TAPLAND, YOU, Zone::hand(YOU));
    assert_eq!(play(&mut g, land, land), (true, Vec::new()));
    let mut g = game();
    put(&mut g, &UNTAPPER, YOU, Zone::BATTLEFIELD);
    let land = put(&mut g, &FILLER, YOU, Zone::hand(YOU));
    assert_eq!(play(&mut g, land, land), (false, Vec::new()));
}
