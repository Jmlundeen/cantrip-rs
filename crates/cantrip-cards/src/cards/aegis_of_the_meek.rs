use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

/// "{1}, {T}: Target 1/1 creature gets +1/+2 until end of turn." (CR 115.1).
fn target_1_1_creature_gets_target_0(q: &dyn Query, _: &Asker, t: Target) -> bool {
    matches!(
        t, Target::Object(id) if targets::permanent(q, id) && q.chars(id).is(CardType::Creature)
            && q.chars(id).power == Some(1) && q.chars(id).toughness == Some(1)
    )
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        ability(
                cost("{1}").and(TAP),
                effects::target_creature_gets::<1, 2>,
                "{1}, {T}: Target 1/1 creature gets +1/+2 until end of turn.",
            )
            .targets(&[target(target_1_1_creature_gets_target_0)]),
    ],
    ..oracle::AEGIS_OF_THE_MEEK
};
