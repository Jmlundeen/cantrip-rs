use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

/// "{G}: Exile target card from a graveyard. If it was a creature card, put a +1/+1 counter on this creature and you gain
/// 1 life." The card's type is read before it leaves (CR 400.7, 608.2h). The counter goes on the Ooze only while it is still
/// on the battlefield (CR 122.1); the life is gained either way, the two being separate instructions (CR 608.2c). With the
/// target illegal on resolution the core does not resolve the ability at all (CR 608.2b).
fn exile_a_card(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(card) = targets::object(g, item, 0) else { return Ok(()) };
    let creature = g.chars(card).is(CardType::Creature);
    g.exile(card);
    if creature {
        if targets::permanent(g, item.object) {
            g.add_counters(item.object, CounterKind::PlusOne, 1);
        }
        effects::gain_life(g, item, 1)?;
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[ability(
        cost("{G}"),
        exile_a_card,
        "{G}: Exile target card from a graveyard. If it was a creature card, put a +1/+1 counter on this creature and you gain 1 life.",
    )
    .targets(&[targets::CARD_IN_A_GRAVEYARD])],
    ..oracle::SCAVENGING_OOZE
};
