use cantrip_api::*;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, effects::tap_and_stun::<1>, "When this creature enters, tap target creature an opponent controls and put a stun counter on it. (If a permanent with a stun counter would become untapped, remove one from it instead.)")
            .targets(&[targets::CREATURE_AN_OPPONENT_CONTROLS]),
        // Renew is an ability word (CR 207.2c): the ability is an ordinary activated ability of the card in the graveyard.
        ability(cost("{2}{U}").and(EXILE_THIS), effects::tap_and_stun::<1>, "Renew \u{2014} {2}{U}, Exile this card from your graveyard: Tap target creature an opponent controls and put a stun counter on it. Activate only as a sorcery.")
            .from(ZoneKind::Graveyard)
            .sorcery_speed()
            .targets(&[targets::CREATURE_AN_OPPONENT_CONTROLS]),
    ],
    ..oracle::CONSTRICTOR_SAGE
};
