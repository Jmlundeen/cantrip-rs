use cantrip_api::*;
use cantrip_mechanics::{evoke, targets};
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_oracle as oracle;

fn white_card(q: &dyn Query, asker: &Asker, id: ObjectId) -> bool {
    q.chars(id).colors.has(q.words(asker.source).color(Color::White))
}

/// "Other target creature" (CR 115.1): a creature on the battlefield that is not this one.
fn other_creature(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    matches!(t, Target::Object(id) if id != asker.source
        && is_live(q, id)
        && q.is_on_battlefield(id)
        && q.chars(id).is(CardType::Creature))
}

/// Exile up to one other target creature; its controller gains life equal to its power, as it last existed.
fn exile_and_compensate(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(creature) = targets::object(g, item, 0) {
        let power = g.chars(creature).power.unwrap_or(0) as i32;
        let controller = g.controller(creature);
        g.exile(creature);
        if power > 0 {
            g.gain_life(controller, power);
        }
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        evoke::sacrifice_trigger("When Solitude enters, if its evoke cost was paid, its controller sacrifices it."),
        triggered(THIS_ENTERS, exile_and_compensate, "When Solitude enters, exile up to one other target creature. That creature's controller gains life equal to its power.")
            .targets(&[TargetSpec { min: 0, max: 1, filter: other_creature }]),
    ],
    // Evoke: exile a white card from your hand rather than pay this spell's mana cost (CR 118.9).
    ways: &[evoke::way(evoke::exile_from_hand(white_card), "Evoke - Exile a white card from your hand")],
    ..oracle::SOLITUDE
};
