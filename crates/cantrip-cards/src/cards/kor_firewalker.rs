use cantrip_api::*;
use cantrip_mechanics::effects::ask_may;
use cantrip_mechanics::protection;
use cantrip_oracle as oracle;

const GAIN: Question = Question::new("GainLife", "gain 1 life?", "A red spell was cast", "Gain 1 life", "Don't gain life");

/// "Whenever a player casts a red spell" (CR 603.2, 601.2i): any player's, this permanent's controller's own included, judged
/// on the spell as it stands on the stack once cast. "Red" is a word of this permanent's text (CR 612.2). The trigger goes on
/// the stack above the spell and resolves first (the ruling).
fn a_player_casts_a_red_spell(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    match event {
        Event::SpellCast { spell, .. } if q.chars(*spell).colors.has(q.words(me.source).color(Color::Red)) => Fired::PLAIN,
        _ => None,
    }
}

/// "... you may gain 1 life." (CR 119.3): asked of this ability's controller as it resolves.
fn may_gain_one_life(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if ask_may(g, item.controller, item.object, Purpose::Ask(&GAIN))? {
        g.gain_life(item.controller, 1);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    statics: &[protection::from::<{ Color::Red as u8 }>("Protection from red")],
    abilities: &[triggered(
        when_asked(EventMask::SPELL_CAST, a_player_casts_a_red_spell),
        may_gain_one_life,
        "Whenever a player casts a red spell, you may gain 1 life.",
    )],
    ..oracle::KOR_FIREWALKER
};
