use cantrip_api::*;
use cantrip_oracle as oracle;

/// "You may activate abilities of creatures you control as though those creatures had haste" (CR 302.6): a creature its
/// controller controls, Shang-Chi included.
fn creature_you_control(q: &dyn Query, asker: &Asker, creature: ObjectId) -> bool {
    q.controller(creature) == asker.controller && q.chars(creature).is(CardType::Creature)
}

pub static CARD: CardDef = CardDef {
    abilities: &[mana_ability(
        TAP,
        add_any_color(2).only_for(ManaRestriction::CREATURE_ABILITIES),
        "{T}: Add two mana of any one color. Spend this mana only to activate abilities of creature sources.",
    )],
    activate_as_though_haste: &[creature_you_control],
    ..oracle::SHANG_CHI_MASTER_OF_KUNG_FU
};
