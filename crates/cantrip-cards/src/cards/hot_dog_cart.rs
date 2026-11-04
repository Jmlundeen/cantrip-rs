use cantrip_api::*;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_mechanics::tokens;
use cantrip_oracle as oracle;

/// "When this artifact enters, create a Food token." (CR 111.10b)
fn create_food(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create_food(g, item.controller)
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, create_food, "When this artifact enters, create a Food token. (It's an artifact with \"{2}, {T}, Sacrifice this token: You gain 3 life.\")"),
        mana_ability(TAP, add_any_color(1), "{T}: Add one mana of any color."),
    ],
    ..oracle::HOT_DOG_CART
};

pub static EXTRA: &[&CardDef] = &[&tokens::FOOD];
