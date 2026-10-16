use cantrip_api::*;
use cantrip_oracle as oracle;

/// "Sacrifice an artifact": any artifact its controller has, itself included.
fn artifact_you_control(q: &dyn Query, asker: &Asker, id: ObjectId) -> bool {
    q.chars(id).is(CardType::Artifact) && q.controller(id) == asker.controller
}

/// "Sacrifice an artifact: Add {C}{C}." A mana ability whose sacrifice comes with the activation.
pub static CARD: CardDef = CardDef {
    abilities: &[mana_ability(sacrifice(1, artifact_you_control), add_n(Mana::C, 2), "Sacrifice an artifact: Add {C}{C}.")],
    ..oracle::KRARK_CLAN_IRONWORKS
};
