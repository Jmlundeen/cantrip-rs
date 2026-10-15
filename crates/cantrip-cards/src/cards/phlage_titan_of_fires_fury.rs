use cantrip_api::*;
use cantrip_mechanics::escape;
use cantrip_mechanics::targets;
use cantrip_mechanics::triggers;
use cantrip_oracle as oracle;

/// "It deals 3 damage to any target and you gain 3 life."
fn bolt_and_gain(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if g.target_legal(item, 0) {
        g.damage(item.object, item.targets[0], 3);
        g.gain_life(item.controller, 3);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        escape::sacrifice_unless_escaped("When Phlage enters, sacrifice it unless it escaped."),
        AbilityDef::Triggered(TriggeredDef {
            text: "When Phlage enters, it deals 3 damage to any target and you gain 3 life.",
            listens: EventMask::ZONE_CHANGE,
            matches: triggers::this_enters,
            targets: &[targets::ANY],
            effect: bolt_and_gain,
            ..TriggeredDef::BLANK
        }),
        AbilityDef::Triggered(TriggeredDef {
            text: "Whenever Phlage attacks, it deals 3 damage to any target and you gain 3 life.",
            listens: EventMask::ATTACKS,
            matches: triggers::this_attacks,
            targets: &[targets::ANY],
            effect: bolt_and_gain,
            ..TriggeredDef::BLANK
        }),
    ],
    ways: &[escape::way(mana(0, 2, 0, 0, 2, 0), 5, "Escape—{R}{R}{W}{W}, Exile five other cards from your graveyard")],
    ..oracle::PHLAGE_TITAN_OF_FIRES_FURY
};
