use cantrip_api::*;
use cantrip_mechanics::{damage_divided, targets, tokens};
use cantrip_oracle as oracle;

/// "Magma Opus deals 4 damage divided as you choose among any number of targets. Tap two target permanents. Create a 4/4
/// blue and red Elemental creature token. Draw two cards." Any number includes zero (CR 107.1c, 601.2d); the damage is
/// divided among the first slot's targets, the second slot's two permanents are tapped, and each sentence ends before the
/// next (CR 608.2c).
fn magma_opus(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    damage_divided(g, item);
    g.end_sentence();
    let permanents = g.state().spec_targets(item, 1).filter_map(|slot| targets::object(g, item, slot)).collect::<Vec<ObjectId>>();
    g.tap_together(&permanents);
    g.end_sentence();
    tokens::create(g, item.controller, &tokens::ELEMENTAL_4_4_BLUE_RED, 1)?;
    g.end_sentence();
    g.draw(item.controller, 2);
    Ok(())
}

/// "{U/R}{U/R}, Discard this card: Create a Treasure token." (CR 111.10a).
fn create_treasure_token(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create_treasure(g, item.controller)
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::ANY.between(0, TargetSpec::ANY_NUMBER), targets::PERMANENT.between(2, 2)], magma_opus),
    abilities: &[
        ability(
            cost("{U/R}{U/R}").and(DISCARD_THIS),
            create_treasure_token,
            "{U/R}{U/R}, Discard this card: Create a Treasure token.",
        )
        .from(ZoneKind::Hand),
    ],
    divide: Some(Divide { total: 4, reads: None }),
    ..oracle::MAGMA_OPUS
};

pub static EXTRA: &[&CardDef] = &[&tokens::ELEMENTAL_4_4_BLUE_RED, &tokens::TREASURE];
