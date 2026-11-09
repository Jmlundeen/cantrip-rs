use cantrip_api::*;
use cantrip_mechanics::{effects, damage_target, destroy_target, targets};
use cantrip_oracle as oracle;

/// Bedazzle: "Destroy target nonbasic land. Bedazzle deals 2 damage to target opponent or planeswalker." Each target is judged as the
/// spell resolves, and one that has become illegal leaves the other still affected (CR 608.2b, the ruling).
fn bedazzle(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    destroy_target(g, item, 0);
    damage_target(g, item, 1, 2);
    Ok(())
}

pub static BEDECK: CardDef = CardDef { spell: spell(&[targets::CREATURE], effects::target_creature_gets::<3, -3>), ..oracle::BEDECK };

pub static BEDAZZLE: CardDef = CardDef {
    spell: spell(
        &[
            targets::NONBASIC_LAND.labelled("Nonbasic land to destroy"),
            targets::OPPONENT_OR_PLANESWALKER.labelled("Opponent or planeswalker to be dealt 2 damage"),
        ],
        bedazzle,
    ),
    ..oracle::BEDAZZLE
};

pub static CARD: CardDef = CardDef { split: Some((&BEDECK, &BEDAZZLE)), ..oracle::BEDECK_BEDAZZLE };

pub static EXTRA: &[&CardDef] = &[&BEDECK, &BEDAZZLE];
