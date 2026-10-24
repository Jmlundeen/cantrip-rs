use cantrip_api::*;
use cantrip_oracle as oracle;

/// "As Three Tree City enters, choose a creature type" (CR 614.1c, 614.12a), kept on the permanent.
fn enters(g: &mut dyn Game, entering: &mut Entering) -> Result<(), Suspend> {
    let ask = RequestKind::ChooseSubtype { source: entering.object, choices: Subtype::CREATURE_TYPES.to_vec(), purpose: WordUse::Chosen };
    if let Answer::Subtype(chosen) = g.ask(entering.controller, ask)? {
        entering.chosen_subtype = Some(chosen);
    }
    Ok(())
}

/// "Add an amount of mana of that color equal to the number of creatures you control of the chosen type" (CR 106.5):
/// the color is named as the ability is activated, the creatures are counted as it resolves, after its {2} is paid.
fn add_for_chosen_type(g: &mut dyn Game, asker: &Asker, color: Option<Color>) -> Result<(), Suspend> {
    let Some(chosen) = g.state().chosen_subtype(g.linked_to(asker.source)) else { return Ok(()) };
    let count = g
        .permanents_of(asker.controller)
        .into_iter()
        .filter(|&id| {
            let c = g.chars(id);
            c.is(CardType::Creature) && c.has_subtype(chosen)
        })
        .count();
    if count > 0 {
        g.add_mana(asker.controller, color.unwrap_or(Color::White).mana(), count as u32);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    enters: Some(enters),
    abilities: &[
        mana_ability(TAP, add(Mana::C), "{T}: Add {C}."),
        mana_ability(cost("{2}, {T}"), add_any_color(1).by(add_for_chosen_type), "{2}, {T}: Choose a color. Add an amount of mana of that color equal to the number of creatures you control of the chosen type."),
    ],
    ..oracle::THREE_TREE_CITY
};
