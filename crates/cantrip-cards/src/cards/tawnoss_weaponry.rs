use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        ability(
                cost("{2}").and(TAP),
                effects::target_creature_gets_while_source_tapped::<1, 1>,
                "{2}, {T}: Target creature gets +1/+1 for as long as this artifact remains tapped.",
            )
            .targets(&[targets::CREATURE]),
    ],
    may_not_untap: true,
    ..oracle::TAWNOSS_WEAPONRY
};
