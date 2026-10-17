use cantrip_api::*;
use cantrip_oracle as oracle;

/// "{T}: Add two mana in any combination of colors. Spend this mana only to cast Elemental spells or activate abilities of
/// Elemental sources." Each mana is a color chosen as the ability resolves, repeats allowed (CR 106.1b); it pays only for
/// casting a spell that is an Elemental on the stack, of any card type, or for activating an ability of a source that is
/// an Elemental - a permanent, or a card in a hand or graveyard (its ruling; CR 106.6). "Elemental" is a word of its text,
/// read through its words (CR 612).
fn add_two(g: &mut dyn Game, asker: &Asker, _: Option<Color>) -> Result<(), Suspend> {
    let elemental = g.words(asker.source).subtype(Subtype::Elemental);
    let restriction = ManaRestriction::Subtypes(SubtypeSpells::of(elemental).or_abilities());
    for color in cantrip_mechanics::choose_colors(g, asker.controller, asker.source, ColorSet(0b11111), 2)? {
        g.add_restricted_mana(asker.controller, color.mana(), 1, restriction);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[mana_ability(
        TAP,
        add_by(add_two).yields_any(2).only_for(ManaRestriction::Subtypes(SubtypeSpells::of(Subtype::Elemental).or_abilities())),
        "{T}: Add two mana in any combination of colors. Spend this mana only to cast Elemental spells or activate abilities of Elemental sources.",
    )],
    ..oracle::FLAMEBRAIDER
};
