use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

/// "{T}: Tap target creature without flying that's attacking you. It doesn't untap during its controller's untap step for as long as this land remains tapped." (CR 608.2, 611.2a, 701.26a).
fn tap_target_creature_without_flying(
    g: &mut dyn Game,
    item: &StackItem,
) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    effects::tap_targets(g, item)?;
    g.end_sentence();
    if targets::permanent(g, creature) {
        g.add_effect(item.object, creature, EffectKind::DoesntUntap, Duration::WhileSourceTapped);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        ability(
                TAP,
                tap_target_creature_without_flying,
                "{T}: Tap target creature without flying that's attacking you. It doesn't untap during its controller's untap step for as long as this land remains tapped.",
            )
            .targets(&[targets::CREATURE_WITHOUT_FLYING_ATTACKING_YOU]),
    ],
    may_not_untap: true,
    ..oracle::ICE_FLOE
};
