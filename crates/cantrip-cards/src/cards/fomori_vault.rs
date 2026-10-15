use cantrip_api::*;
use cantrip_mechanics::library::{look_at_top, Rest};
use cantrip_mechanics::mana;
use cantrip_oracle as oracle;

fn any_card(_: &dyn Query, _: &Asker, _: ObjectId) -> bool {
    true
}

/// Look at the top X cards of your library, where X is the number of artifacts you control as this resolves. Put one of
/// those cards into your hand and the rest on the bottom of your library in a random order. No artifacts is looking at
/// nothing, which does nothing.
fn look(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let player = item.controller;
    let artifacts = g.permanents_of(player).into_iter().filter(|&id| g.chars(id).is(CardType::Artifact)).count();
    look_at_top(g, player, item.object, artifacts, (1, 1), false, |_| true, Rest::BottomRandom)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana::tap_for(mana::add_c, Mana::C, "{T}: Add {C}."),
        AbilityDef::Activated(ActivatedDef {
            text: "{3}, {T}, Discard a card: Look at the top X cards of your library, where X is the number of artifacts you control. Put one of those cards into your hand and the rest on the bottom of your library in a random order.",
            cost: Cost { mana: mana(3, 0, 0, 0, 0, 0), tap: true, discard: Some((1, any_card)), ..Cost::NONE },
            effect: look,
            ..ActivatedDef::BLANK
        }),
    ],
    ..oracle::FOMORI_VAULT
};
