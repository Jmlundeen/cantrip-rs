use cantrip_api::*;
use cantrip_mechanics::{library, top_of_library, triggers};
use cantrip_oracle as oracle;

/// "Whenever this creature or another Elemental you control enters, look at the top card of your library. If it's a land card, you may put it onto the battlefield tapped. If you don't put the card onto the battlefield, put it into your hand." (CR 608.2, 400.7).
fn look_top_card_library_if(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let looked = top_of_library(g, item.controller, 1);
    g.look_at(item.controller, &looked);
    g.end_sentence();
    for original in looked {
        let entered = library::may_put_land_onto_battlefield_tapped(g, item.controller, item.object, original)?;
        g.end_sentence();
        if !entered {
            let card = g.found_after_move(original).unwrap_or(original);
            g.return_to_hand(&[card]);
        }
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            when_asked(
                EventMask::ZONE_CHANGE,
                triggers::this_or_another_of_subtype_you_control_enters::<
                    { Subtype::Elemental as u16 },
                >,
            ),
            look_top_card_library_if,
            "Whenever this creature or another Elemental you control enters, look at the top card of your library. If it's a land card, you may put it onto the battlefield tapped. If you don't put the card onto the battlefield, put it into your hand.",
        ),
    ],
    ..oracle::RISEN_REEF
};
