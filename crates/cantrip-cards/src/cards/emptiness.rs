use cantrip_api::*;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_mechanics::{evoke, mana, targets};
use cantrip_oracle as oracle;

/// The intervening "if {W}{W} was spent to cast it" (CR 603.4): checked as it enters and again as the ability resolves.
fn white_white_spent(q: &dyn Query, asker: &Asker) -> bool {
    mana::two_spent(q, asker.source, Mana::W)
}

/// The intervening "if {B}{B} was spent to cast it" (CR 603.4).
fn black_black_spent(q: &dyn Query, asker: &Asker) -> bool {
    mana::two_spent(q, asker.source, Mana::B)
}

/// "Return target creature card with mana value 3 or less from your graveyard to the battlefield." Nothing happens to a
/// target that is no longer legal (CR 608.2b); its one target is required, so an ability whose target is gone is removed
/// before it gets here and the target is always still there.
fn reanimate(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    targets::object(g, item, 0).map(|card| g.put_onto_battlefield(card, item.controller, false)).transpose()?;
    Ok(())
}

/// "Put three -1/-1 counters on up to one target creature."
fn three_minus_counters(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(creature) = targets::object(g, item, 0) {
        g.add_counters(creature, CounterKind::MinusOne, 3);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    ways: &[evoke::way(
        Cost::mana(ManaCost {
            generic: 0,
            symbols: [0; 6],
            hybrid: [Some((Mana::W, Mana::B)), Some((Mana::W, Mana::B)), None, None, None, None],
            phyrexian: [0; 5],
            x: 0,
        }),
        "Evoke {W/B}{W/B}",
    )],
    abilities: &[
        evoke::sacrifice_trigger("When Emptiness enters, if its evoke cost was paid, its controller sacrifices it."),
        triggered(
            THIS_ENTERS,
            reanimate,
            "When this creature enters, if {W}{W} was spent to cast it, return target creature card with mana value 3 or less from your graveyard to the battlefield.",
        )
        .targets(&[targets::CREATURE_CARD_WITH_MANA_VALUE_3_OR_LESS_IN_YOUR_GRAVEYARD])
        .condition(white_white_spent),
        triggered(
            THIS_ENTERS,
            three_minus_counters,
            "When this creature enters, if {B}{B} was spent to cast it, put three -1/-1 counters on up to one target creature.",
        )
        .targets(&[targets::CREATURE.between(0, 1)])
        .condition(black_black_spent),
    ],
    ..oracle::EMPTINESS
};
