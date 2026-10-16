use cantrip_api::*;
use cantrip_mechanics::power_up;
use cantrip_mechanics::triggers::THIS_ATTACKS;
use cantrip_oracle as oracle;

/// Cybernetic Senses (an ability word, CR 207.2c): "Whenever Viv Vision attacks, draw a card if her power is 4 or
/// greater." The power is read as the ability resolves (CR 608.2c), as she last was if she has left (CR 608.2h).
fn cybernetic_senses(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if g.chars(item.object).power.unwrap_or(0) >= 4 {
        g.draw(item.controller, 1);
    }
    Ok(())
}

/// "Put two +1/+1 counters on Viv Vision."
fn two_counters(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.add_counters(item.object, CounterKind::PlusOne, 2);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ATTACKS, cybernetic_senses, "Cybernetic Senses \u{2014} Whenever Viv Vision attacks, draw a card if her power is 4 or greater."),
        power_up::ability(
            cost("{7}"),
            two_counters,
            "Power-up \u{2014} {7}: Put two +1/+1 counters on Viv Vision. (Activate each power-up ability only once. Reduce the cost by her mana cost if she entered this turn.)",
        ),
    ],
    ..oracle::VIV_VISION_TEEN_SYNTHEZOID
};
