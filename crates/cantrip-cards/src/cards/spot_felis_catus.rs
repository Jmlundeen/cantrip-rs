use cantrip_api::*;
use cantrip_mechanics::targets::filters;
use cantrip_mechanics::{destroy_target, targets, tokens, triggers};
use cantrip_oracle as oracle;

/// "create a Food token" (CR 111.10b, 111.10f): a Food artifact token under the ability's controller.
fn make_food(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &tokens::FOOD, 1)?;
    Ok(())
}

/// "{4}, Sacrifice a Food: Destroy target artifact" (CR 701.8a): the cost is paid as the ability is activated, so the Food
/// is already gone when the target is destroyed; nothing if the artifact is no longer a legal target (CR 608.2b).
fn destroy_artifact(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    destroy_target(g, item, 0);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            triggers::NONTOKEN_LEGENDARY_PERMANENT_YOU_CONTROL_ENTERS,
            make_food,
            "Whenever Spot or another nontoken legendary permanent you control enters, create a Food token.",
        ),
        ability(
            cost("{4}").and(Cost::sacrifice(1, filters::food)),
            destroy_artifact,
            "{4}, Sacrifice a Food: Destroy target artifact.",
        )
        .targets(&[targets::ARTIFACT]),
    ],
    ..oracle::SPOT_FELIS_CATUS
};

pub static EXTRA: &[&CardDef] = &[&tokens::FOOD];
