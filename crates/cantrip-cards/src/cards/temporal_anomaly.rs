use cantrip_api::*;
use cantrip_mechanics::{library, targets, tokens};
use cantrip_oracle as oracle;

/// "Mill four cards. You may put a permanent card from among them into your hand." (CR 701.17a, 701.17c, 110.4a):
/// `library::mill_then_may_take_into_hand`. Then a new sentence, "Create a tapped Planet land token named New Planet
/// with '{T}: Add one mana of any color.'" (CR 111.1, 111.4).
fn mill_and_create(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    library::mill_then_may_take_into_hand(g, item.controller, item.object, 4, "permanent", targets::is_permanent_card)?;
    g.end_sentence();
    tokens::create_by_rule(g, item.controller, &tokens::NEW_PLANET, 1, true)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add(Mana::C), "{T}: Add {C}."),
        ability(
            cost("{3}{G}").and(TAP).and(SACRIFICE_THIS),
            mill_and_create,
            "{3}{G}, {T}, Sacrifice this land: Mill four cards. You may put a permanent card from among them into your hand. Create a tapped Planet land token named New Planet with \"{T}: Add one mana of any color.\"",
        ),
    ],
    ..oracle::TEMPORAL_ANOMALY
};

pub static EXTRA: &[&CardDef] = &[&tokens::NEW_PLANET];
