use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "It's a Skeleton in addition to its other types and has no abilities": what the creature enters as (CR 614.12, 205.1b, 613.1d,
/// 613.1f), from the moment it enters, so an ability that triggers "when this enters" is gone before it can trigger (the ruling).
/// It keeps its other types and subtypes and loses every ability it had (one it gains later is kept, CR 613.7).
static AS_A_SKELETON_WITH_NO_ABILITIES: EntersAs = EntersAs { subtypes: &[Subtype::Skeleton], loses_other_abilities: true, ..EntersAs::NONE };

/// "Return target creature card from your graveyard to the battlefield. It's a Skeleton in addition to its other types and has no
/// abilities." (CR 400.3): with the target gone the ability does not resolve (CR 608.2b); a card a replacement effect sent
/// elsewhere never became a permanent (CR 400.7).
fn reanimate_as_a_skeleton_with_no_abilities(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(card) = targets::object(g, item, 0) else { return Ok(()) };
    g.put_onto_battlefield_as(card, item.controller, item, &AS_A_SKELETON_WITH_NO_ABILITIES)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[ability(
        TAP,
        reanimate_as_a_skeleton_with_no_abilities,
        "{T}: Return target creature card from your graveyard to the battlefield. It's a Skeleton in addition to its other types and has no abilities. Activate only as a sorcery.",
    )
    .sorcery_speed()
    .targets(&[targets::CREATURE_CARD_IN_YOUR_GRAVEYARD])],
    ..oracle::XU_IFIT_OSTEOHARMONIST
};
