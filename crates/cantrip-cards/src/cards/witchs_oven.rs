use cantrip_api::*;
use cantrip_mechanics::targets::filters;
use cantrip_mechanics::tokens;
use cantrip_oracle as oracle;

/// "Create a Food token. If the sacrificed creature's toughness was 4 or greater, create two Food tokens instead": the
/// sacrificed creature is read as it last was on the battlefield (CR 608.2h), so its toughness counts as it stood when it
/// was sacrificed, with every effect on it. The two Food tokens are created together (CR 111.1).
fn bake(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let big = g.cost_objects(item).first().is_some_and(|&c| g.chars(c).toughness.unwrap_or(0) >= 4);
    tokens::create(g, item.controller, &tokens::FOOD, if big { 2 } else { 1 })?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[ability(
        TAP.and(sacrifice(1, filters::creature)),
        bake,
        "{T}, Sacrifice a creature: Create a Food token. If the sacrificed creature's toughness was 4 or greater, create two Food tokens instead.",
    )],
    ..oracle::WITCHS_OVEN
};

pub static EXTRA: &[&CardDef] = &[&tokens::FOOD];
