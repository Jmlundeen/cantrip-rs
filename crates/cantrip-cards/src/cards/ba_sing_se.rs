use cantrip_api::*;
use cantrip_mechanics::earthbend;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    enters: Some(cantrip_mechanics::lands::enters_tapped_unless_you_control_basic_land),
    abilities: &[
        mana_ability(TAP, add(Mana::G), "{T}: Add {G}."),
        ability(cost("{2}{G}, {T}"), earthbend::effect::<2>, "{2}{G}, {T}: Earthbend 2. Activate only as a sorcery.")
            .targets(&[earthbend::TARGET])
            .sorcery_speed(),
    ],
    delayed: &[earthbend::RETURN],
    ..oracle::BA_SING_SE
};
