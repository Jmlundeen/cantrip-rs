use cantrip_api::*;
use cantrip_mechanics::clockwork;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        clockwork::end_of_combat(
            "At end of combat, if this creature attacked or blocked this combat, remove a +1/+0 counter from it.",
        ),
        clockwork::recharge::<7>(
            "{X}, {T}: Put up to X +1/+0 counters on this creature. This ability can't cause the total number of +1/+0 counters on this creature to be greater than seven. Activate only during your upkeep.",
        ),
    ],
    enters: Some(clockwork::enters::<7>),
    ..oracle::CLOCKWORK_BEAST
};
