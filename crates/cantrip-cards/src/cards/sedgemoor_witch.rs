use cantrip_api::*;
use cantrip_mechanics::tokens;
use cantrip_mechanics::triggers::{MAGECRAFT, ward_life};
use cantrip_oracle as oracle;

fn create_a_pest(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &tokens::PEST, 1)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        ward_life::<3>("Ward\u{2014}Pay 3 life. (Whenever this creature becomes the target of a spell or ability an opponent controls, counter it unless that player pays 3 life.)"),
        triggered(MAGECRAFT, create_a_pest, "Magecraft \u{2014} Whenever you cast or copy an instant or sorcery spell, create a 1/1 black and green Pest creature token with \"When this token dies, you gain 1 life.\""),
    ],
    ..oracle::SEDGEMOOR_WITCH
};

pub static EXTRA: &[&CardDef] = &[&tokens::PEST];
