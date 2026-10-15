use cantrip_api::*;
use cantrip_mechanics::mana;
use cantrip_oracle as oracle;

/// "{T}: Add {C}{C}. Ancient Tomb deals 2 damage to you." A mana ability: the damage is dealt as it resolves.
fn add_cc_and_hurt(g: &mut dyn Game, asker: &Asker, _: Option<Color>) -> Result<(), Suspend> {
    g.add_mana(asker.controller, Mana::C, 2);
    g.damage(asker.source, Target::Player(asker.controller), 2);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[mana::tap_for_n(add_cc_and_hurt, Mana::C, 2, "{T}: Add {C}{C}. Ancient Tomb deals 2 damage to you.")],
    ..oracle::ANCIENT_TOMB
};
