use cantrip_api::*;
use cantrip_mechanics::statics;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[mana_ability(
        TAP,
        add_any_color(2).only_for(ManaRestriction::CREATURE_ABILITIES),
        "{T}: Add two mana of any one color. Spend this mana only to activate abilities of creature sources.",
    )],
    activate_as_though_haste: &[statics::creatures_you_control_as_though_haste],
    ..oracle::SHANG_CHI_MASTER_OF_KUNG_FU
};
