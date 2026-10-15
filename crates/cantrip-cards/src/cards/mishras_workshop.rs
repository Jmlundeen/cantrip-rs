use cantrip_api::*;
use cantrip_oracle as oracle;

/// {T}: Add {C}{C}{C}. Spend this mana only to cast artifact spells (CR 106.6).
fn add_restricted(g: &mut dyn Game, asker: &Asker, _: Option<Color>) -> Result<(), Suspend> {
    g.add_restricted_mana(asker.controller, Mana::C, 3, ManaRestriction::ArtifactSpells);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[AbilityDef::Activated(ActivatedDef {
        text: "{T}: Add {C}{C}{C}. Spend this mana only to cast artifact spells.",
        cost: Cost { tap: true, ..Cost::NONE },
        mana: Some(add_restricted),
        adds: ManaPool([0, 0, 0, 0, 0, 3]),
        restricted: Some(ManaRestriction::ArtifactSpells),
        ..ActivatedDef::BLANK
    })],
    ..oracle::MISHRAS_WORKSHOP
};
