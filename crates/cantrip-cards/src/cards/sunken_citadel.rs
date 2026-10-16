use cantrip_api::*;
use cantrip_mechanics::choose_color;
use cantrip_oracle as oracle;

/// "This land enters tapped. As it enters, choose a color." (CR 614.1c, 614.12a): both as it enters, the color kept on
/// the permanent (CR 607.2d).
fn enters(g: &mut dyn Game, entering: &mut Entering) -> Result<(), Suspend> {
    entering.tapped = true;
    entering.chosen_color = Some(choose_color(g, entering.controller, entering.object)?);
    Ok(())
}

/// "Mana of the chosen color" (CR 106.5): the one color chosen as it entered, none if none was (a land that became a
/// copy of it without entering), and then the ability adds no mana. "The chosen color" is no color word, so a
/// text-changing effect leaves it as it is (CR 612.2).
fn chosen(q: &dyn Query, asker: &Asker) -> ColorSet {
    q.obj(asker.source).chosen_color.map_or(ColorSet::NONE, ColorSet::of)
}

pub static CARD: CardDef = CardDef {
    enters: Some(enters),
    abilities: &[
        mana_ability(TAP, add_any_color(1), "{T}: Add one mana of the chosen color.").colors_from(chosen),
        mana_ability(TAP, add_any_color(2).only_for(ManaRestriction::LAND_ABILITIES), "{T}: Add two mana of the chosen color. Spend this mana only to activate abilities of land sources.")
            .colors_from(chosen),
    ],
    ..oracle::SUNKEN_CITADEL
};
