use cantrip_api::*;
use cantrip_mechanics::{channel, effects, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add(Mana::W), "{T}: Add {W}."),
        // The card is the source, as it last existed in the hand it was discarded from (CR 113.7a, 608.2h).
        ability(cost("{2}{W}").and(DISCARD_THIS), effects::damage_first_target::<4>, "Channel \u{2014} {2}{W}, Discard this card: It deals 4 damage to target attacking or blocking creature. This ability costs {1} less to activate for each legendary creature you control.")
            .targets(&[targets::ATTACKING_OR_BLOCKING_CREATURE])
            .from(ZoneKind::Hand),
    ],
    own_cost_changes: &[channel::cheaper_per_legendary_creature],
    ..oracle::EIGANJO_SEAT_OF_THE_EMPIRE
};
