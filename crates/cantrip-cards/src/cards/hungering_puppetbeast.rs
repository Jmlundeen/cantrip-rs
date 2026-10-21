use cantrip_api::*;
use cantrip_mechanics::{choose, effects, haste_until_end_of_turn, targets, tokens, trample_until_end_of_turn};
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_oracle as oracle;

/// The sacrifice must be another artifact the activating player controls (CR 118.3, 701.21a).
fn another_artifact(q: &dyn Query, asker: &Asker, id: ObjectId) -> bool {
    id != asker.source && q.chars(id).is(CardType::Artifact) && q.controller(id) == asker.controller
}

/// "When this creature enters, create a Heartwood token." (CR 603.2, 701.7a).
fn create_heartwood(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create_heartwood(g, item.controller)
}

/// "Put a +1/+1 counter on this creature. It gains your choice of trample, hexproof, or haste until end of turn."
/// (CR 122.1, 608.2d, 611.2a, 702.10, 702.11, 702.19).
fn grow(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let permanent = item.object;
    g.add_counters(permanent, CounterKind::PlusOne, 1);
    let card = g.obj(item.object).card;
    match choose::one_of(
        g,
        item.controller,
        item.object,
        (0..3).map(|option| Label::Card(card, option)).collect(),
    )? {
        0 => trample_until_end_of_turn(g, item.object, permanent),
        1 => {
            if targets::permanent(g, permanent) {
                effects::gain_hexproof_until_end_of_turn(g, item.object, permanent);
            }
        }
        _ => haste_until_end_of_turn(g, item.object, permanent),
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, create_heartwood, "When this creature enters, create a Heartwood token."),
        ability(
            cost("{1}").and(sacrifice(1, another_artifact)),
            grow,
            "{1}, Sacrifice another artifact: Put a +1/+1 counter on this creature. It gains your choice of trample, hexproof, or haste until end of turn.",
        ),
    ],
    labels: &["Trample", "Hexproof", "Haste"],
    ..oracle::HUNGERING_PUPPETBEAST
};

pub static EXTRA: &[&CardDef] = &[&tokens::HEARTWOOD];
