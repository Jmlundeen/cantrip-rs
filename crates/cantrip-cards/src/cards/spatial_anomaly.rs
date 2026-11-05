use cantrip_api::*;
use cantrip_mechanics::{targets, tokens};
use cantrip_oracle as oracle;

/// "It deals 2 damage to target creature." (CR 120.2, 608.2b): the target if it is still legal as the ability resolves.
/// Then a new sentence, "Create a tapped Planet land token named New Planet with '{T}: Add one mana of any color.'"
/// (CR 111.1, 111.4), made whether or not the creature was still there.
fn damage_and_create(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    cantrip_mechanics::damage_target(g, item, 0, 2);
    g.end_sentence();
    tokens::create_by_rule(g, item.controller, &tokens::NEW_PLANET, 1, true)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add(Mana::C), "{T}: Add {C}."),
        ability(
            cost("{4}{R}").and(TAP).and(SACRIFICE_THIS),
            damage_and_create,
            "{4}{R}, {T}, Sacrifice this land: It deals 2 damage to target creature. Create a tapped Planet land token named New Planet with \"{T}: Add one mana of any color.\"",
        )
        .targets(&[targets::CREATURE]),
    ],
    ..oracle::SPATIAL_ANOMALY
};

pub static EXTRA: &[&CardDef] = &[&tokens::NEW_PLANET];
