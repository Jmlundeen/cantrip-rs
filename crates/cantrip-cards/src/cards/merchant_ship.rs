use cantrip_api::*;
use cantrip_mechanics::{effects, lands};
use cantrip_oracle as oracle;

/// This creature attacks and isn't blocked (CR 509.1h).
fn unblocked(_: &dyn Query, asker: &Asker, event: &Event) -> Option<Fired> {
    match event {
        Event::Unblocked { attacker, .. } if *attacker == asker.source => Fired::about(Target::Object(*attacker)),
        _ => None,
    }
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(when_asked(EventMask::UNBLOCKED, unblocked), |g, item| effects::gain_life(g, item, 2),
            "Whenever this creature attacks and isn't blocked, you gain 2 life."),
        state_triggered(lands::you_control_no_islands, effects::sacrifice_self, "When you control no Islands, sacrifice this creature."),
    ],
    attack_rules: &[lands::attack_rule_defender_controls::<{ lands::sub(Subtype::Island) }>],
    ..oracle::MERCHANT_SHIP
};
