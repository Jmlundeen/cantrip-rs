use cantrip_api::*;
use cantrip_mechanics::{effects, prevention, statics, tokens};
use cantrip_oracle as oracle;

/// "+1: Until end of turn, Gideon becomes a 5/5 Human Soldier Ally creature with indestructible that's still a planeswalker.
/// Prevent all damage that would be dealt to him this turn." (CR 611.2a, 615.1).
fn gideon_becomes_5_5_human(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let keywords = Keywords::of(&[Keyword::Indestructible]);
    effects::planeswalker_becomes_creature_preventing_damage(g, item, (5, 5), None, &[Subtype::Human, Subtype::Soldier, Subtype::Ally], keywords, 0);
    Ok(())
}

/// "0: Create a 2/2 white Knight Ally creature token." (CR 111.1).
fn create_2_2_white_knight(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &tokens::KNIGHT_ALLY, 1)?;
    Ok(())
}

pub static EMBLEM: CardDef = CardDef {
    name: "Gideon, Ally of Zendikar emblem",
    has_cost: false,
    text: "Creatures you control get +1/+1.",
    statics: &[statics::CREATURES_YOU_CONTROL_GET_PLUS_ONE],
    ..CardDef::BLANK
};

/// "−4: You get an emblem with "Creatures you control get +1/+1."" (CR 114.2).
fn get_emblem_creatures_control_get(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create_emblem(g, item.controller, &EMBLEM);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        ability(
            loyalty(1),
            gideon_becomes_5_5_human,
            "+1: Until end of turn, Gideon becomes a 5/5 Human Soldier Ally creature with indestructible that's still a planeswalker. Prevent all damage that would be dealt to him this turn.",
        ),
        ability(loyalty(0), create_2_2_white_knight, "0: Create a 2/2 white Knight Ally creature token."),
        ability(
            loyalty(-4),
            get_emblem_creatures_control_get,
            "\u{2212}4: You get an emblem with \"Creatures you control get +1/+1.\"",
        ),
    ],
    granted_preventions: &[prevention::ANY_DAMAGE.text("Prevent all damage that would be dealt to him this turn.")],
    ..oracle::GIDEON_ALLY_OF_ZENDIKAR
};

pub static EXTRA: &[&CardDef] = &[&tokens::KNIGHT_ALLY, &EMBLEM];
