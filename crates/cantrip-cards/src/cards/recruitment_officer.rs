use cantrip_api::*;
use cantrip_mechanics::library::{self, Rest};
use cantrip_mechanics::select;
use cantrip_oracle as oracle;

/// "Look at the top four cards of your library. You may reveal a creature card with mana value 3 or less from among them and put
/// it into your hand. Put the rest on the bottom of your library in a random order." (CR 701.20a, 401.4)
fn look_at_top_four(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    library::look_at_top_select(
        g,
        item.controller,
        item.object,
        4,
        select::creature_cards(select::up_to(1)).with(select::mana_value_at_most(3)),
        true,
        Rest::BottomRandom,
    )?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[ability(
        cost("{3}{W}"),
        look_at_top_four,
        "{3}{W}: Look at the top four cards of your library. You may reveal a creature card with mana value 3 or less from among them and put it into your hand. Put the rest on the bottom of your library in a random order.",
    )],
    ..oracle::RECRUITMENT_OFFICER
};
