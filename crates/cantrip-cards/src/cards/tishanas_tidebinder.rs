use cantrip_api::*;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_oracle as oracle;

/// "Up to one target activated or triggered ability" (CR 115.1): an ability on the stack. Mana abilities never are.
fn ability_filter(_: &dyn Query, _: &Asker, t: Target) -> bool {
    matches!(t, Target::Ability(_))
}

/// Counter the ability (CR 701.6); if it was an ability of an artifact, creature or planeswalker, that permanent loses
/// all abilities for as long as this creature remains on the battlefield (CR 611.2b, 613.1f). The permanent's type is
/// read before the ability is countered, and the removal is made only if the ability really left the stack.
fn counter_and_silence(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if item.target_count == 0 || !g.target_legal(item, 0) {
        return Ok(());
    }
    let Target::Ability(serial) = item.targets[0] else { return Ok(()) };
    let Some(at) = g.state().stack.iter().position(|i| i.serial == serial) else { return Ok(()) };
    let countered = g.state().stack[at];
    let source = countered.object;
    let qualifies = is_live(g, source) && g.is_on_battlefield(source) && {
        let c = g.chars(source);
        c.is(CardType::Artifact) || c.is(CardType::Creature) || c.is(CardType::Planeswalker)
    };
    g.counter_item(source, at);
    if qualifies && !g.state().stack.iter().any(|i| i.serial == serial) {
        g.add_effect(item.object, source, EffectKind::LoseAllAbilities, Duration::WhileSourceOnBattlefield);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(THIS_ENTERS, counter_and_silence, "When this creature enters, counter up to one target activated or triggered ability. If an ability of an artifact, creature, or planeswalker is countered this way, that permanent loses all abilities for as long as this creature remains on the battlefield.")
        .targets(&[TargetSpec { min: 0, max: 1, filter: ability_filter }])],
    ..oracle::TISHANAS_TIDEBINDER
};
