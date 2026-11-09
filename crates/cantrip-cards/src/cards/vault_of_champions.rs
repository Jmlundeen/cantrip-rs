use cantrip_api::*;
use cantrip_mechanics::lands;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    enters: Some(lands::enters_tapped_unless_two_or_more_opponents),
    abilities: &[mana_ability(TAP, add_one_of(&[Color::White, Color::Black]), "{T}: Add {W} or {B}.")],
    ..oracle::VAULT_OF_CHAMPIONS
};
