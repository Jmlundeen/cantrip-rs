use cantrip_api::*;
use cantrip_mechanics::{effects, targets, triggers};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    // Renew is an ability word (CR 207.2c): an ordinary activated ability of the card in the graveyard.
    abilities: &[
        triggered(triggers::THIS_ENTERS, effects::plus_one_counter_on_target, "When this creature enters, put a +1/+1 counter on target creature.")
            .targets(&[targets::CREATURE]),
        ability(cost("{3}{G}").and(EXILE_THIS), effects::plus_one_counter_on_target_then_double, "Renew \u{2014} {3}{G}, Exile this card from your graveyard: Put a +1/+1 counter on target creature, then double the number of +1/+1 counters on that creature. Activate only as a sorcery.")
            .from(ZoneKind::Graveyard)
            .sorcery_speed()
            .targets(&[targets::CREATURE]),
    ],
    ..oracle::SAGE_OF_THE_FANG
};
