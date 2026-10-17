use cantrip_api::*;
use cantrip_mechanics::{effects, triggers};
use cantrip_mechanics::{Animation, animate, mana};
use cantrip_oracle as oracle;

/// "This land becomes a 3/3 green and white Llama creature until end of turn. It's still a land." (CR 613.1d, 613.4b, 613.1e)
fn becomes_creature(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let words = g.words(item.object);
    let animation = Animation {
        power: 3,
        toughness: 3,
        colors: Some(words.colors(ColorSet(ColorSet::of(Color::Green).0 | ColorSet::of(Color::White).0))),
        types: TypeSet::NONE,
        subtypes: &[words.subtype(Subtype::Llama)],
        all_creature_types: false,
        keywords: Keywords::NONE,
        ability: None,
    };
    animate(g, item.object, item.object, &animation, Duration::EndOfTurn);
    Ok(())
}

/// "Whenever this land attacks, other creatures you control get +1/+1 until end of turn." The creatures are those its controller
/// controls as it resolves, not this land, and none that enters later (CR 611.2c).
fn pump_others(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::pump_other_creatures_you_control(g, item, 1, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    enters: Some(mana::enters_tapped),
    abilities: &[
        mana_ability(TAP, add_one_of(&[Color::Green, Color::White]), "{T}: Add {G} or {W}."),
        ability(cost("{2}{G}{W}"), becomes_creature, "{2}{G}{W}: This land becomes a 3/3 green and white Llama creature until end of turn. It's still a land."),
        triggered(triggers::THIS_ATTACKS, pump_others, "Whenever this land attacks, other creatures you control get +1/+1 until end of turn."),
    ],
    ..oracle::RESTLESS_PRAIRIE
};
