use cantrip_api::*;
use cantrip_mechanics::flashback;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// Put two +1/+1 counters on target creature. It gains vigilance, trample, lifelink, indestructible, and haste until
/// end of turn.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    g.add_counters(creature, CounterKind::PlusOne, 2);
    g.add_effect(
        item.object,
        creature,
        EffectKind::AddKeywords(Keywords::of(&[
            Keyword::Vigilance,
            Keyword::Trample,
            Keyword::Lifelink,
            Keyword::Indestructible,
            Keyword::Haste,
        ])),
        Duration::EndOfTurn,
    );
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: Some(SpellDef { targets: &[targets::CREATURE], effect }),
    ways: &[flashback::way(mana(2, 1, 0, 0, 1, 0), "Flashback {2}{R}{W}")],
    ..oracle::ANGELFIRE_IGNITION
};
