use cantrip_api::*;
use cantrip_mechanics::{choose, lands, select};
use cantrip_oracle as oracle;

fn creature_or_spacecraft(q: &dyn Query, choice: &Choice, _: u32) -> Verdict {
    Verdict::each(choice, |card| q.chars(card).is(CardType::Creature) || q.chars(card).has_subtype(Subtype::Spacecraft))
}

/// "A creature or Spacecraft card", up to one (CR 205.2a), judged as the card in the hand.
const CREATURE_OR_SPACECRAFT_CARD: Selection = Selection::of(select::up_to(1), &[cantrip_api::select::part(&(creature_or_spacecraft as SelectFn), 0, "")]).called("creature or Spacecraft");

/// "+1/+1 counters on it if it's a creature and ... charge counters on it if it's a Spacecraft".
const IF_CREATURE_OR_SPACECRAFT: &[(CounterKind, EnteringIs)] =
    &[(CounterKind::PlusOne, EnteringIs::Type(CardType::Creature)), (CounterKind::Charge, EnteringIs::Subtype(Subtype::Spacecraft))];

/// "Strange new worlds - {6}, {T}: You may put a creature or Spacecraft card from your hand onto the battlefield. It enters
/// with X additional +1/+1 counters on it if it's a creature and X additional charge counters on it if it's a Spacecraft,
/// where X is the number of differently named lands you control."
///
/// The card is chosen from the whole hand shown, and may be left there (CR 608.2d, 101.4); it is a creature or Spacecraft
/// card as the card in the hand. X is counted as the card is put onto the battlefield, from the lands its controller controls
/// and no one else's (CR 201.2a-b), and the counters are extra ones it enters with (CR 614.1c, 122.1). "If it's a creature"
/// and "if it's a Spacecraft" are asked of the permanent as it would exist on the battlefield (CR 614.12), after the copy
/// effect it enters with (CR 616.1c): an artifact that March of the Machines makes a creature gets the +1/+1 counters, and a
/// Phyrexian Metamorph entering as a copy of a noncreature Spacecraft gets the charge counters alone. Zero counters are not
/// put at all (CR 122.6).
fn strange_new_worlds(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    let hand = g.state().zone(Zone::hand(you)).to_vec();
    let Some(card) = choose::select(g, you, item.object, Purpose::PutOntoBattlefield, hand, CREATURE_OR_SPACECRAFT_CARD)?.first().copied() else {
        return Ok(());
    };
    let x = lands::differently_named_lands(g, you) as i32;
    g.put_onto_battlefield_with_counters_if(card, you, false, CountersIf { amount: x, parts: IF_CREATURE_OR_SPACECRAFT })?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[ability(
        cost("{6}").and(TAP),
        strange_new_worlds,
        "Strange new worlds — {6}, {T}: You may put a creature or Spacecraft card from your hand onto the battlefield. It enters with X additional +1/+1 counters on it if it's a creature and X additional charge counters on it if it's a Spacecraft, where X is the number of differently named lands you control.",
    )],
    ..oracle::CHIEF_MONTGOMERY_SCOTT
};
