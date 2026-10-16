use cantrip_api::*;
use cantrip_oracle as oracle;

fn forest_you_control(q: &dyn Query, asker: &Asker, id: ObjectId) -> bool {
    q.chars(id).has_subtype(q.words(asker.source).subtype(Subtype::Forest)) && q.controller(id) == asker.controller
}

/// "Add three mana in any combination of {R} and/or {G}" (CR 106.1b): each mana is a color of its own choosing.
fn add_three(g: &mut dyn Game, asker: &Asker, _: Option<Color>) -> Result<(), Suspend> {
    let choices = ColorSet(ColorSet::of(Color::Red).0 | ColorSet::of(Color::Green).0);
    if let Answer::Colors(colors) = g.ask(asker.controller, RequestKind::ChooseColors { source: asker.source, choices, count: 3 })? {
        for color in colors {
            g.add_mana(asker.controller, color.mana(), 1);
        }
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[mana_ability(TAP.and(Cost::sacrifice(1, forest_you_control)), add_by(add_three).yields_any(3), "{T}, Sacrifice a Forest: Add three mana in any combination of {R} and/or {G}.")],
    ..oracle::ORCISH_LUMBERJACK
};
