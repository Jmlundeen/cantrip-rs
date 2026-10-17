use cantrip_api::*;
use cantrip_oracle as oracle;

/// The eight creature types the land offers, words of its text (CR 612: a text-changing effect changes what is offered).
const OFFERED: [Subtype; 8] =
    [Subtype::Elemental, Subtype::Elf, Subtype::Faerie, Subtype::Giant, Subtype::Goblin, Subtype::Kithkin, Subtype::Merfolk, Subtype::Treefolk];

/// "As this land enters, choose Elemental, Elf, Faerie, Giant, Goblin, Kithkin, Merfolk, or Treefolk" (CR 614.12a): one of
/// the eight, asked of its controller as it enters and kept on it (CR 607.2a).
fn choose_type(g: &mut dyn Game, entering: &mut Entering) -> Result<(), Suspend> {
    let words = g.words(entering.object);
    let choices = OFFERED.iter().map(|&t| words.subtype(t)).collect();
    cantrip_mechanics::chosen_type::choose_type_among(g, entering, choices)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    enters: Some(choose_type),
    abilities: &[
        mana_ability(TAP, add(Mana::C), "{T}: Add {C}."),
        // "Of the chosen type" is the type chosen as it entered, or none if none was (a copy that never entered): then the
        // mana pays for nothing (CR 607.2a, 106.6).
        mana_ability(
            TAP,
            add_any_color(1).only_for(ManaRestriction::Subtypes(SubtypeSpells::CHOSEN_TYPE_SPELLS_OR_ABILITIES)),
            "{T}: Add one mana of any color. Spend this mana only to cast a spell of the chosen type or activate an ability of a source of the chosen type.",
        ),
    ],
    ..oracle::ECLIPSED_REALMS
};
