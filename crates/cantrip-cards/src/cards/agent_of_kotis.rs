use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    // Renew is an ability word (CR 207.2c): the ability is an ordinary activated ability of the card in the graveyard.
    abilities: &[ability(cost("{3}{U}").and(EXILE_THIS), effects::two_plus_one_counters_on_target, "Renew \u{2014} {3}{U}, Exile this card from your graveyard: Put two +1/+1 counters on target creature. Activate only as a sorcery.")
        .from(ZoneKind::Graveyard)
        .sorcery_speed()
        .targets(&[targets::CREATURE])],
    ..oracle::AGENT_OF_KOTIS
};
