use cantrip_api::*;
use cantrip_mechanics::attach::equip;
use cantrip_mechanics::pump;
use cantrip_oracle as oracle;

/// Roll a six-sided die (CR 706). That creature gets +X/+0 until end of turn, where X is the result. Then if it has the
/// greatest power or is tied for greatest power among creatures on the battlefield, draw a card. A creature that has
/// left the battlefield gets nothing and has no power to compare.
fn roll_and_pump(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let x = g.roll_die(item.controller, 6).result as i16;
    let Target::Object(creature) = item.subject else { return Ok(()) };
    if !is_live(g, creature) || !g.is_on_battlefield(creature) {
        return Ok(());
    }
    pump(g, item.object, creature, x, 0);
    let power_of = |g: &dyn Game, id: ObjectId| g.chars(id).power.unwrap_or(0);
    let mine = power_of(g, creature);
    let greatest = g
        .state()
        .battlefield()
        .iter()
        .copied()
        .filter(|&id| g.chars(id).is(CardType::Creature))
        .all(|id| power_of(g, id) <= mine);
    if greatest {
        g.draw(item.controller, 1);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        AbilityDef::Triggered(TriggeredDef {
            text: "Whenever equipped creature attacks, roll a six-sided die. That creature gets +X/+0 until end of turn, where X is the result. Then if it has the greatest power or is tied for greatest power among creatures on the battlefield, draw a card.",
            listens: EventMask::ATTACKS,
            matches: cantrip_mechanics::triggers::equipped_creature_attacks_subject,
            effect: roll_and_pump,
            ..TriggeredDef::BLANK
        }),
        equip(mana(3, 0, 0, 0, 0, 0), "Equip {3}"),
    ],
    ..oracle::STRENGTH_TESTING_HAMMER
};
