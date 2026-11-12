use cantrip_api::*;
use cantrip_mechanics::triggers::subtype_you_control_deal_combat_damage_to_player;
use cantrip_mechanics::{lands, TAP_AN_UNTAPPED_CREATURE, effects, tokens};
use cantrip_oracle as oracle;

/// "You draw a card and create a 1/1 red Officer creature token." The card first, then the token (CR 608.2c).
fn draw_a_card_and_create_an_officer(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.draw(item.controller, 1);
    tokens::create(g, item.controller, &tokens::OFFICER, 1)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            when_asked(EventMask::DAMAGE, subtype_you_control_deal_combat_damage_to_player::<{ lands::sub(Subtype::Officer) }>),
            draw_a_card_and_create_an_officer,
            "Whenever one or more Officers you control deal combat damage to a player, you draw a card and create a 1/1 red Officer creature token.",
        )
        .sums_per_subject(),
        ability(
            cost("{2}").and(TAP_AN_UNTAPPED_CREATURE),
            effects::this_cant_be_blocked_this_turn,
            "{2}, Tap an untapped creature you control: Captain Janeway can't be blocked this turn.",
        ),
    ],
    ..oracle::CAPTAIN_JANEWAY_ENDGAME
};

pub static EXTRA: &[&CardDef] = &[&tokens::OFFICER];
