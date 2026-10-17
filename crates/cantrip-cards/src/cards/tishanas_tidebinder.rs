use cantrip_api::*;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_mechanics::{counter_target_ability, targets};
use cantrip_oracle as oracle;

/// Counter the ability (CR 701.6); if it was an ability of an artifact, creature or planeswalker, that permanent loses
/// all abilities for as long as this creature remains on the battlefield (CR 611.2b, 613.1f). Countering an ability changes
/// nothing about its source, whose type is read as it is now, and the removal is made only if the ability really left the
/// stack.
fn counter_and_silence(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(countered) = counter_target_ability(g, item, 0) else { return Ok(()) };
    let source = countered.object;
    let qualifies = is_live(g, source) && g.is_on_battlefield(source) && {
        let c = g.chars(source);
        c.is(CardType::Artifact) || c.is(CardType::Creature) || c.is(CardType::Planeswalker)
    };
    if qualifies && !g.state().stack.iter().any(|i| i.serial == countered.serial) {
        g.add_effect(item.object, source, EffectKind::LoseAllAbilities, Duration::WhileSourceOnBattlefield);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(THIS_ENTERS, counter_and_silence, "When this creature enters, counter up to one target activated or triggered ability. If an ability of an artifact, creature, or planeswalker is countered this way, that permanent loses all abilities for as long as this creature remains on the battlefield.")
        .targets(&[up_to_one(targets::activated_or_triggered_ability_filter)])],
    ..oracle::TISHANAS_TIDEBINDER
};
