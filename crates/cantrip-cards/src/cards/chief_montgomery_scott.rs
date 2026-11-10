use cantrip_api::*;
use cantrip_mechanics::{hand, lands};
use cantrip_oracle as oracle;

/// "Strange new worlds — {6}, {T}: You may put a creature or Spacecraft card from your hand onto the battlefield. It enters
/// with X additional +1/+1 counters on it if it's a creature and X additional charge counters on it if it's a Spacecraft,
/// where X is the number of differently named lands you control."
///
/// The card is chosen from the whole hand shown, and may be left there (CR 608.2d, 101.4). X is counted as the card is put
/// onto the battlefield, from the lands its controller controls and no one else's (CR 201.2a-b), and the counters are extra
/// ones it enters with (CR 614.1c, 122.1): +1/+1 ones when the card is a creature, charge ones when it is a Spacecraft, both
/// when it is both. Zero counters are not put at all (CR 122.6).
fn strange_new_worlds(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    hand::may_put_onto_battlefield_with_counters(
        g,
        you,
        item.object,
        |g, card| {
            let chars = g.chars(card);
            chars.is(CardType::Creature) || chars.has_subtype(Subtype::Spacecraft)
        },
        |g, card| {
            let x = lands::differently_named_lands(g, you) as i32;
            let chars = g.chars(card);
            let mut counters = Vec::new();
            if x > 0 {
                if chars.is(CardType::Creature) {
                    counters.push((CounterKind::PlusOne, x));
                }
                if chars.has_subtype(Subtype::Spacecraft) {
                    counters.push((CounterKind::Charge, x));
                }
            }
            counters
        },
    )?;
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
