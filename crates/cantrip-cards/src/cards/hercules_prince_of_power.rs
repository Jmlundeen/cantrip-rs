use cantrip_api::*;
use cantrip_mechanics::power_up;
use cantrip_oracle as oracle;

/// "Put a +1/+1 counter on Hercules. He gains vigilance, indestructible, and haste until end of turn." Neither, once he
/// has left the battlefield (CR 400.7).
fn counter_and_keywords(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.add_counters(item.object, CounterKind::PlusOne, 1);
    g.add_effect(
        item.object,
        item.object,
        EffectKind::AddKeywords(Keywords::of(&[Keyword::Vigilance, Keyword::Indestructible, Keyword::Haste])),
        Duration::EndOfTurn,
    );
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[power_up::ability(
        cost("{4}{G}"),
        counter_and_keywords,
        "Power-up \u{2014} {4}{G}: Put a +1/+1 counter on Hercules. He gains vigilance, indestructible, and haste until end of turn. (Activate each power-up ability only once. Reduce the cost by his mana cost if he entered this turn.)",
    )],
    ..oracle::HERCULES_PRINCE_OF_POWER
};
