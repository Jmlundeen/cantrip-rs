use cantrip_api::*;
use cantrip_mechanics::triggers::you_cast_instant_or_sorcery;
use cantrip_mechanics::{choose, effects, targets};
use cantrip_oracle as oracle;

const TEMUR: u8 = 0;
const SULTAI: u8 = 1;

/// "As this enchantment enters, choose Temur or Sultai" (CR 614.12a, 607.2f).
fn enters(g: &mut dyn Game, entering: &mut Entering) -> Result<(), Suspend> {
    choose::option_as_enters(g, entering, &CARD, 0..2)
}

/// "• Temur — Whenever you cast an instant or sorcery spell" (CR 603.2): only while Temur was chosen.
fn temur_cast(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    (choose::chosen_option(q, me, &CARD) == Some(TEMUR)).then(|| you_cast_instant_or_sorcery(q, me, event)).flatten()
}

/// "• Sultai — You may play lands from your graveyard" holds only while Sultai was chosen (CR 607.2f).
fn sultai_chosen(q: &dyn Query, me: &Asker) -> bool {
    choose::chosen_option(q, me, &CARD) == Some(SULTAI)
}

pub static CARD: CardDef = CardDef {
    enters: Some(enters),
    labels: &["Temur", "Sultai"],
    abilities: &[triggered(
        when_asked(EventMask::SPELL_CAST, temur_cast),
        effects::target_player_mills::<4>,
        "Temur \u{2014} Whenever you cast an instant or sorcery spell, target player mills four cards.",
    )
    .targets(&[targets::PLAYER])],
    // Each land played this way still uses a land play (CR 305.1, 305.2).
    play_rules: &[PlayRule::While(sultai_chosen, &[PlayRule::LandsFromGraveyard])],
    ..oracle::GLACIERWOOD_SIEGE
};
