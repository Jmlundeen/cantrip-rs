use cantrip_api::*;
use cantrip_mechanics::{effects, lands, permanents_with_subtype_you_control, triggers};
use cantrip_oracle as oracle;

/// "When this enters, draw a card for each Shrine you control" (CR 121.1, 608.2c): counted as the ability resolves, the Shrines
/// its controller controls then, this one included if it is still on the battlefield, and drawn together; the type is read
/// through the words of the Oasis's text (CR 612).
fn draw_for_each_shrine(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let shrine = g.words(item.object).subtype(Subtype::Shrine);
    let shrines = permanents_with_subtype_you_control(g, item.controller, shrine);
    g.draw(item.controller, shrines as u32);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(triggers::THIS_ENTERS, draw_for_each_shrine, "When The Spirit Oasis enters, draw a card for each Shrine you control."),
        triggered(
            when_asked(EventMask::ZONE_CHANGE, triggers::another_of_subtype_you_control_enters::<{ lands::sub(Subtype::Shrine) }>),
            |g, item| effects::draw(g, item, 1),
            "Whenever another Shrine you control enters, draw a card.",
        ),
    ],
    ..oracle::THE_SPIRIT_OASIS
};
