use cantrip_api::*;
use cantrip_mechanics::effects::plus_one_counters_on_first_target_that_gains_until_end_of_turn;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[ability(
        cost("{1}{U}").and(EXILE_THIS),
        plus_one_counters_on_first_target_that_gains_until_end_of_turn::<1, { Keywords::of(&[Keyword::Flying]).0 }>,
        "{1}{U}, Exile this card from your graveyard: Put a +1/+1 counter on target creature. It gains flying until end of turn. Activate only as a sorcery.",
    )
    .from(ZoneKind::Graveyard)
    .targets(&[targets::CREATURE])
    .sorcery_speed()],
    ..oracle::BEETLE_LEGACY_CRIMINAL
};
