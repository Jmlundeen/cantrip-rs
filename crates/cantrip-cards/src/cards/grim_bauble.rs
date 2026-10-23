use cantrip_api::*;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, effects::target_creature_gets::<-2, -2>, "When this artifact enters, target creature an opponent controls gets -2/-2 until end of turn.")
            .targets(&[targets::CREATURE_AN_OPPONENT_CONTROLS]),
        ability(
            cost("{2}{B}, {T}").and(SACRIFICE_THIS),
            |g, item| effects::surveil(g, item, 2),
            "{2}{B}, {T}, Sacrifice this artifact: Surveil 2.",
        ),
    ],
    ..oracle::GRIM_BAUBLE
};
