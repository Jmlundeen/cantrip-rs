use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_oracle as oracle;

/// "{T}: Add {G}." and "Put a -1/-1 counter on this creature: Untap this creature." The counter is a cost (CR 118.3, 602.2b),
/// put on as the ability is activated, so a Druid whose toughness it brings to 0 dies (CR 704.5f) before the untap resolves.
pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add(Mana::G), "{T}: Add {G}."),
        ability(
            put_counters(CounterKind::MinusOne, 1),
            effects::untap_self,
            "Put a -1/-1 counter on this creature: Untap this creature.",
        ),
    ],
    ..oracle::DEVOTED_DRUID
};
