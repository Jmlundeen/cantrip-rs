use cantrip_api::*;
use cantrip_mechanics::triggers::THIS_DIES;
use cantrip_oracle as oracle;

/// Set the base power and toughness of creatures that dealt damage to this object this turn (CR 613.4b, 611.2c).
fn set_damaging_creatures(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let creatures: Vec<ObjectId> = g.state().battlefield().iter().copied().filter(|&id| {
        g.chars(id).is(CardType::Creature)
            && g.history().this_turn().iter().any(|record| matches!(record.event,
                Event::Damage { source, target: Target::Object(target), .. } if source == id && target == item.object))
    }).collect();
    instruction(g, |g| {
        for &creature in &creatures {
            g.add_effect(item.object, creature, EffectKind::SetPt(0, 2), Duration::Indefinite);
        }
    });
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(THIS_DIES, set_damaging_creatures,
        "When this creature dies, the base power and toughness of each creature that dealt damage to it this turn become 0/2.")],
    ..oracle::BRINE_HAG
};
