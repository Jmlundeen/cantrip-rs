use cantrip_api::*;
use cantrip_mechanics::tokens::{self, ROBOT_2_2};
use cantrip_mechanics::triggers::{creature_you_control_deals_combat_damage_to_player, YOUR_END_STEP};
use cantrip_mechanics::{effects, history};
use cantrip_oracle as oracle;

/// Void (intervening "if", CR 603.4): "a nonland permanent left the battlefield this turn or a spell was warped this
/// turn", checked as the end step begins and again as the ability resolves.
fn void(q: &dyn Query, _: &Asker) -> bool {
    history::void(q)
}

/// "Create a 2/2 colorless Robot artifact creature token."
fn create_robot(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &ROBOT_2_2, 1)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        // "One or more creatures ... deal combat damage to a player" (CR 603.2c): one trigger for each player the batch
        // damaged, however many of the creatures dealt it.
        triggered(
            when_asked(EventMask::DAMAGE, creature_you_control_deals_combat_damage_to_player),
            effects::draw_a_card_and_lose_1_life,
            "Whenever one or more creatures you control deal combat damage to a player, you draw a card and lose 1 life.",
        )
        .sums_per_subject(),
        triggered(
            YOUR_END_STEP,
            create_robot,
            "Void \u{2014} At the beginning of your end step, if a nonland permanent left the battlefield this turn or a spell was warped this turn, create a 2/2 colorless Robot artifact creature token.",
        )
        .condition(void),
    ],
    ..oracle::ELEGY_ACOLYTE
};

pub static EXTRA: &[&CardDef] = &[&ROBOT_2_2];
