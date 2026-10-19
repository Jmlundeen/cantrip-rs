use cantrip_api::*;
use cantrip_mechanics::triggers::opponent_casts_spell_with_mana_value_at_most;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

/// The Adventure (CR 715): Return target nonland permanent to its owner's hand.
pub static STEAM_CLEAN: CardDef = CardDef {
    spell: spell(&[targets::NONLAND_PERMANENT], effects::return_first_target_to_hand),
    ..oracle::STEAM_CLEAN
};

/// "This creature deals 1 damage to that player": the player who cast the spell (`StackItem::subject`).
fn one_to_that_player(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.damage(item.object, item.subject, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        when(EventMask::SPELL_CAST, opponent_casts_spell_with_mana_value_at_most::<3>),
        one_to_that_player,
        "Whenever an opponent casts a spell with mana value 3 or less, this creature deals 1 damage to that player.",
    )],
    adventure: Some(&STEAM_CLEAN),
    ..oracle::SCALDING_VIPER
};

pub static EXTRA: &[&CardDef] = &[&STEAM_CLEAN];
