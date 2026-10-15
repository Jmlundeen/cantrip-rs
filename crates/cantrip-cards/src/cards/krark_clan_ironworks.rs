use cantrip_api::*;
use cantrip_mechanics::mana;
use cantrip_oracle as oracle;

/// "Sacrifice an artifact": any artifact its controller has, itself included.
fn artifact_you_control(q: &dyn Query, asker: &Asker, id: ObjectId) -> bool {
    q.chars(id).is(CardType::Artifact) && q.controller(id) == asker.controller
}

/// "Sacrifice an artifact: Add {C}{C}." A mana ability whose sacrifice comes with the activation.
pub static CARD: CardDef = CardDef {
    abilities: &[AbilityDef::Activated(ActivatedDef {
        text: "Sacrifice an artifact: Add {C}{C}.",
        cost: Cost { permanents: Some(PermanentCost { count: 1, filter: artifact_you_control, to_hand: false }), ..Cost::NONE },
        mana: Some(mana::add_cc),
        adds: ManaPool([0, 0, 0, 0, 0, 2]),
        ..ActivatedDef::BLANK
    })],
    ..oracle::KRARK_CLAN_IRONWORKS
};
