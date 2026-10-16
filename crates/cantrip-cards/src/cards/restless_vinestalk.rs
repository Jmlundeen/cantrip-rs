use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_mechanics::triggers;
use cantrip_mechanics::{Animation, animate};
use cantrip_oracle as oracle;

fn becomes_creature(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let words = g.words(item.object);
    let animation = Animation {
        power: 5,
        toughness: 5,
        colors: Some(words.colors(ColorSet(ColorSet::of(Color::Green).0 | ColorSet::of(Color::Blue).0))),
        subtype: Some(words.subtype(Subtype::Plant)),
        keywords: Keywords::of(&[Keyword::Trample]),
        ability: None,
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
        mana_ability(TAP, add_one_of(&[Color::Green, Color::Blue]), "{T}: Add {G} or {U}."),
        ability(cost("{3}{U}{G}"), becomes_creature, "{3}{G}{U}: Until end of turn, this land becomes a 5/5 green and blue Plant creature with trample. It's still a land."),
        triggered(triggers::THIS_ATTACKS, three_three, "Whenever this land attacks, up to one other target creature has base power and toughness 3/3 until end of turn.")
            .targets(&[TargetSpec { min: 0, max: 1, filter: other_creature, label: "" }]),
    ],
    ..oracle::RESTLESS_VINESTALK
};
