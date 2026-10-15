use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_mechanics::targets::filters;
use cantrip_oracle as oracle;

/// "At the beginning of each opponent's draw step" (CR 503.1a, 603.2b): the draw step of a turn that is not its
/// controller's.
fn opponents_draw_step(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::StepBegan { step: Step::Draw } if q.state().turn.active != q.controller(source) => Fired::PLAIN,
        _ => None,
    }
}

/// "You draw a card."
fn draw_a_card(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.draw(item.controller, 1);
    Ok(())
}

/// The 1/1 blue Illusion creature token the +1 makes.
pub static ILLUSION: CardDef = CardDef {
    name: "Illusion",
    has_cost: false,
    colors: Some(ColorSet::of(Color::Blue)),
    types: TypeSet::of(&[CardType::Creature]),
    subtypes: SubtypeSet::of(&[Subtype::Illusion]),
    power: Some(1),
    toughness: Some(1),
    ..CardDef::BLANK
};

/// "+1: Create a 1/1 blue Illusion creature token."
fn make_illusion(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    cantrip_mechanics::tokens::create(g, item.controller, &ILLUSION, 1).map(|_| ())
}

/// One of "for each opponent, up to one target artifact or creature that player controls" (CR 115.1, 115.3): a
/// artifact or creature an opponent controls, when no earlier slot chose one that opponent controls. A slot is one
/// target each, so there is a slot for each opponent a game can have (up to three) and each is optional; the filter
/// leaves a slot nothing to choose once every opponent has one.
fn one_per_opponent(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    let Target::Object(id) = t else { return false };
    targets::permanent(q, id)
        && filters::artifact_or_creature(q, asker, id)
        && q.controller(id) != asker.controller
        && !q.chosen_targets().iter().any(|c| matches!(c, Target::Object(e) if q.controller(*e) == q.controller(id)))
}

const UP_TO_ONE: TargetSpec = TargetSpec { min: 0, max: 1, filter: one_per_opponent };

/// "\u{2212}2: For each opponent, return up to one target artifact or creature that player controls to its owner's
/// hand." The targets still legal are returned together (CR 608.2b).
fn bounce(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let objects = targets::legal_objects(g, item);
    g.return_to_hand(&objects);
    Ok(())
}

/// "\u{2212}6: Draw three cards. Then put X +1/+1 counters on each creature you control, where X is the number of cards
/// in your hand." X is counted after the draw.
fn ultimate(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    g.draw(you, 3);
    let x = g.state().zone(Zone::hand(you)).len() as i16;
    for id in g.permanents_of(you) {
        if g.chars(id).is(CardType::Creature) {
            g.add_counters(id, CounterKind::PlusOne, x);
        }
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        AbilityDef::Triggered(TriggeredDef {
            text: "At the beginning of each opponent's draw step, you draw a card.",
            listens: EventMask::STEP_BEGAN,
            matches: opponents_draw_step,
            effect: draw_a_card,
            ..TriggeredDef::BLANK
        }),
        AbilityDef::Activated(ActivatedDef {
            text: "+1: Create a 1/1 blue Illusion creature token.",
            cost: Cost { loyalty: Some(1), ..Cost::NONE },
            timing: Timing::Sorcery,
            effect: make_illusion,
            ..ActivatedDef::BLANK
        }),
        AbilityDef::Activated(ActivatedDef {
            text: "\u{2212}2: For each opponent, return up to one target artifact or creature that player controls to its owner's hand.",
            cost: Cost { loyalty: Some(-2), ..Cost::NONE },
            timing: Timing::Sorcery,
            targets: &[UP_TO_ONE, UP_TO_ONE, UP_TO_ONE],
            effect: bounce,
            ..ActivatedDef::BLANK
        }),
        AbilityDef::Activated(ActivatedDef {
            text: "\u{2212}6: Draw three cards. Then put X +1/+1 counters on each creature you control, where X is the number of cards in your hand.",
            cost: Cost { loyalty: Some(-6), ..Cost::NONE },
            timing: Timing::Sorcery,
            effect: ultimate,
            ..ActivatedDef::BLANK
        }),
    ],
    ..oracle::THE_THEORIST_JACE_BELEREN
};

pub static EXTRA: &[&CardDef] = &[&ILLUSION];
