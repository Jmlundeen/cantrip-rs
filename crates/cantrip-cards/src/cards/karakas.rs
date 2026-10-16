use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "Target legendary creature": on the battlefield, whoever controls it.
fn legendary_creature(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    (targets::CREATURE.filter)(q, asker, t) && matches!(t, Target::Object(id) if q.chars(id).supertypes.legendary)
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add(Mana::W), "{T}: Add {W}."),
        ability(TAP, cantrip_mechanics::effects::return_first_target_to_hand, "{T}: Return target legendary creature to its owner's hand.")
            .targets(&[target(legendary_creature)]),
    ],
    ..oracle::KARAKAS
};
