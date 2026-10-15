use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_mechanics::triggers::this_enters;
use cantrip_oracle as oracle;

/// Each opponent loses 1 life and you gain 1 life.
fn drain(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.each_opponent_loses_life(item.controller, 1);
    g.gain_life(item.controller, 1);
    Ok(())
}

/// "If you're the monarch" (CR 603.4): asked as the ability would trigger and again as it resolves.
fn you_are_monarch(q: &dyn Query, asker: &Asker) -> bool {
    q.state().player(asker.controller).monarch
}

/// Return up to one target creature card from your graveyard to your hand, if it is still a legal target.
fn return_it(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(card) = targets::object(g, item, 0) {
        g.return_to_hand(&[card]);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        AbilityDef::Triggered(TriggeredDef {
            text: "When this enchantment enters, you become the monarch.",
            listens: EventMask::ZONE_CHANGE,
            matches: this_enters,
            effect: cantrip_mechanics::effects::become_monarch,
            ..TriggeredDef::BLANK
        }),
        AbilityDef::Triggered(TriggeredDef {
            text: "Whenever a creature you control dies, each opponent loses 1 life and you gain 1 life.",
            listens: EventMask::ZONE_CHANGE,
            matches: cantrip_mechanics::triggers::creature_you_control_dies,
            effect: drain,
            look_back: true,
            ..TriggeredDef::BLANK
        }),
        AbilityDef::Triggered(TriggeredDef {
            text: "At the beginning of your end step, if you're the monarch, return up to one target creature card from your graveyard to your hand.",
            listens: EventMask::STEP_BEGAN,
            matches: cantrip_mechanics::triggers::your_end_step,
            targets: &[TargetSpec { min: 0, max: 1, ..cantrip_mechanics::targets::CREATURE_CARD_IN_YOUR_GRAVEYARD }],
            condition: Some(you_are_monarch),
            effect: return_it,
            ..TriggeredDef::BLANK
        }),
    ],
    ..oracle::GRAVE_VENERATIONS
};
