use cantrip_api::*;
use cantrip_mechanics::your_turn;
use cantrip_oracle as oracle;

/// "Add X mana in any combination of {U} and/or {R}, where X is Vivi Ornitier's power" (CR 106.1b, 605.3c-d): X is
/// its power as the mana is added, and each mana is a color of its own choosing. With no power it adds nothing, and
/// asks nothing.
fn add_x_mana(g: &mut dyn Game, asker: &Asker, _: Option<Color>) -> Result<(), Suspend> {
    let x = g.chars(asker.source).power.unwrap_or(0).max(0) as u8;
    if x == 0 {
        return Ok(());
    }
    let choices = ColorSet(ColorSet::of(Color::Blue).0 | ColorSet::of(Color::Red).0);
    if let Answer::Colors(colors) = g.ask(asker.controller, RequestKind::ChooseColors { source: asker.source, choices, count: x })? {
        for color in colors {
            g.add_mana(asker.controller, color.mana(), 1);
        }
    }
    Ok(())
}

fn grow_and_ping(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.add_counters(item.object, CounterKind::PlusOne, 1);
    let hits: Vec<(ObjectId, Target, u16)> = g.state().opponents(item.controller).map(|p| (item.object, Target::Player(p), 1)).collect();
    g.damage_together(&hits);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(Cost::NONE, add_by(add_x_mana), "{0}: Add X mana in any combination of {U} and/or {R}, where X is Vivi Ornitier's power. Activate only during your turn and only once each turn.")
            .condition(your_turn)
            .once_each_turn(),
        triggered(cantrip_mechanics::triggers::NONCREATURE_SPELL_YOU_CAST, grow_and_ping, "Whenever you cast a noncreature spell, put a +1/+1 counter on Vivi Ornitier and it deals 1 damage to each opponent."),
    ],
    ..oracle::VIVI_ORNITIER
};
