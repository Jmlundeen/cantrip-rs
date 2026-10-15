use cantrip_api::*;
use cantrip_mechanics::mana::tap_for_either;
use cantrip_mechanics::targets;
use cantrip_mechanics::triggers;
use cantrip_mechanics::{Animation, animate};
use cantrip_oracle as oracle;

fn becomes_creature(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let animation = Animation {
        power: 5,
        toughness: 5,
        colors: ColorSet(ColorSet::of(Color::Green).0 | ColorSet::of(Color::Blue).0),
        subtype: Subtype::Plant,
        keywords: Keywords::of(&[Keyword::Trample]),
    };
    animate(g, item.object, item.object, &animation, Duration::EndOfTurn);
    Ok(())
}

/// "Up to one other target creature" (CR 115.1).
fn other_creature(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    (targets::CREATURE.filter)(q, asker, t) && t != Target::Object(asker.source)
}

/// It has base power and toughness 3/3 until end of turn (CR 613.4b).
fn three_three(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if item.target_count > 0
        && let Some(creature) = targets::object(g, item, 0)
    {
        g.add_effect(item.object, creature, EffectKind::SetPt(3, 3), Duration::EndOfTurn);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    enters: Some(cantrip_mechanics::mana::enters_tapped),
    abilities: &[
        tap_for_either(Color::Green, Color::Blue, "{T}: Add {G} or {U}."),
        AbilityDef::Activated(ActivatedDef {
            text: "{3}{G}{U}: Until end of turn, this land becomes a 5/5 green and blue Plant creature with trample. It's still a land.",
            cost: Cost { mana: mana(3, 0, 1, 0, 0, 1), ..Cost::NONE },
            effect: becomes_creature,
            ..ActivatedDef::BLANK
        }),
        AbilityDef::Triggered(TriggeredDef {
            text: "Whenever this land attacks, up to one other target creature has base power and toughness 3/3 until end of turn.",
            listens: EventMask::ATTACKS,
            matches: triggers::this_attacks,
            targets: &[TargetSpec { min: 0, max: 1, filter: other_creature }],
            effect: three_three,
            ..TriggeredDef::BLANK
        }),
    ],
    ..oracle::RESTLESS_VINESTALK
};
