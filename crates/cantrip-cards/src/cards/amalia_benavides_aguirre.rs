use cantrip_api::*;
use cantrip_mechanics::explore;
use cantrip_mechanics::triggers::{YOU_GAIN_LIFE, ward_life};
use cantrip_oracle as oracle;

/// "Whenever you gain life, Amalia Benavides Aguirre explores. Then destroy all other creatures if its power is exactly
/// 20." One ability: nobody can respond between the explore and the check (CR 608.2). Amalia explores even if she has left
/// the battlefield (CR 701.44c: last known information, no counter then). Her power is read once, now that she has explored:
/// as she is, or as she last was on the battlefield (CR 113.7a, 608.2h). Every other creature, whoever controls it, is
/// destroyed together (CR 701.8a).
fn explore_then_maybe_destroy(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    explore(g, item.object)?;
    if g.chars(item.object).power != Some(20) {
        return Ok(());
    }
    let others: Vec<ObjectId> = g
        .state()
        .battlefield()
        .iter()
        .copied()
        .filter(|&id| id != item.object && g.chars(id).is(CardType::Creature))
        .collect();
    g.destroy_all(&others);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        ward_life::<3>("Ward\u{2014}Pay 3 life."),
        triggered(
            YOU_GAIN_LIFE,
            explore_then_maybe_destroy,
            "Whenever you gain life, Amalia Benavides Aguirre explores. Then destroy all other creatures if its power is exactly 20.",
        ),
    ],
    ..oracle::AMALIA_BENAVIDES_AGUIRRE
};
