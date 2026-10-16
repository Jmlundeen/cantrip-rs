use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "Target legendary creature": on the battlefield, whoever controls it.
fn legendary_creature(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    (targets::CREATURE.filter)(q, asker, t) && matches!(t, Target::Object(id) if q.chars(id).supertypes.legendary)
}

/// Return target legendary creature to its owner's hand.
fn bounce(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(creature) = targets::object(g, item, 0) {
        g.return_to_hand(&[creature]);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add(Mana::W), "{T}: Add {W}."),
        ability(TAP, bounce, "{T}: Return target legendary creature to its owner's hand.")
            .targets(&[TargetSpec { min: 1, max: 1, filter: legendary_creature }]),
    ],
    ..oracle::KARAKAS
};
