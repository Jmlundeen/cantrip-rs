use cantrip_api::*;
use cantrip_mechanics::{targets, triggers::this_enters, triggers};
use cantrip_oracle as oracle;

/// Up to one target creature can't block this turn. If you're the monarch, creatures can't block this turn.
fn cant_block(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(creature) = targets::object(g, item, 0) {
        g.add_effect(item.object, creature, EffectKind::CantBlock, Duration::EndOfTurn);
    }
    if g.state().player(item.controller).monarch {
        g.creatures_cant_block();
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        AbilityDef::Triggered(TriggeredDef {
            text: "When Aragorn enters, you become the monarch.",
            listens: EventMask::ZONE_CHANGE,
            matches: this_enters,
            effect: cantrip_mechanics::effects::become_monarch,
            ..TriggeredDef::BLANK
        }),
        AbilityDef::Triggered(TriggeredDef {
            text: "Whenever Aragorn attacks, up to one target creature can't block this turn. If you're the monarch, creatures can't block this turn.",
            listens: EventMask::ATTACKS,
            matches: triggers::this_attacks,
            targets: &[TargetSpec { min: 0, ..targets::CREATURE }],
            effect: cant_block,
            ..TriggeredDef::BLANK
        }),
    ],
    ..oracle::ARAGORN_KING_OF_GONDOR
};
