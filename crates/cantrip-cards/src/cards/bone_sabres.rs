use cantrip_api::*;
use cantrip_mechanics::attach::equip;
use cantrip_oracle as oracle;

/// Put four +1/+1 counters on it: on nothing if the creature has since left the battlefield (CR 400.7).
fn four_counters(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Target::Object(creature) = item.subject
        && is_live(g, creature)
        && g.is_on_battlefield(creature)
    {
        g.add_counters(creature, CounterKind::PlusOne, 4);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        AbilityDef::Triggered(TriggeredDef {
            text: "Whenever equipped creature attacks, put four +1/+1 counters on it.",
            listens: EventMask::ATTACKS,
            matches: cantrip_mechanics::triggers::equipped_creature_attacks_subject,
            effect: four_counters,
            ..TriggeredDef::BLANK
        }),
        equip(mana(3, 0, 0, 0, 0, 0), "Equip {3}"),
    ],
    ..oracle::BONE_SABRES
};
